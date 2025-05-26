#![no_std]

extern crate alloc;

use alloc::{format, string::ToString, vec::Vec};
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
    Error, Expr, ItemFn, LitInt, LitStr, Token,
};

/// The default buffer size for the logger.
const DEFAULT_BUFFER_SIZE: &str = "200";

/// Represents the input arguments to the `log!` macro.
struct LogArgs {
    /// The length of the buffer to use for the logger.
    ///
    /// This does not have effect when the literal `str` does
    /// not have value placeholders.
    buffer_len: LitInt,

    /// The literal formatting string passed to the macro.
    ///
    /// The `str` might have value placeholders. While this is
    /// not a requirement, the number of placeholders must
    /// match the number of args.
    format_string: LitStr,

    /// The arguments passed to the macro.
    ///
    /// The arguments represent the values to replace the
    /// placeholders on the format `str`. Valid values must implement
    /// the [`Log`] trait.
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for LogArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Optional buffer length.
        let buffer_len = if input.peek(LitInt) {
            let literal = input.parse()?;
            // Parse the comma after the buffer length.
            input.parse::<Token![,]>()?;
            literal
        } else {
            parse_str::<LitInt>(DEFAULT_BUFFER_SIZE)?
        };

        let format_string = input.parse()?;
        // Check if there are any arguments passed to the macro.
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };

        Ok(LogArgs {
            buffer_len,
            format_string,
            args,
        })
    }
}

/// Companion `log!` macro for `pinocchio-log`.
///
/// The macro automates the creation of a `Logger` object to log a message.
/// It support a limited subset of the [`format!`](https://doc.rust-lang.org/std/fmt/) syntax.
/// The macro parses the format string at compile time and generates the calls to a `Logger`
/// object to generate the corresponding formatted message.
///
/// # Arguments
///
/// - `buffer_len`: The length of the buffer to use for the logger (default to `200`). This is an optional argument.
/// - `format_string`: The literal string to log. This string can contain placeholders `{}` to be replaced by the arguments.
/// - `args`: The arguments to replace the placeholders in the format string. The arguments must implement the `Log` trait.
#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    // Parse the input into a `LogArgs`.
    let LogArgs {
        buffer_len,
        format_string,
        args,
    } = parse_macro_input!(input as LogArgs);
    let parsed_string = format_string.value();

    // Regex pattern to match placeholders in the format string.
    let placeholder_regex = Regex::new(r"\{.*?\}").unwrap();

    let placeholders: Vec<_> = placeholder_regex
        .find_iter(&parsed_string)
        .map(|m| m.as_str())
        .collect();

    // Check if there is an argument for each `{}` placeholder.
    if placeholders.len() != args.len() {
        let arg_message = if args.is_empty() {
            "but no arguments were given".to_string()
        } else {
            format!(
                "but there is {} {}",
                args.len(),
                if args.len() == 1 {
                    "argument"
                } else {
                    "arguments"
                }
            )
        };

        return Error::new_spanned(
            format_string,
            format!(
                "{} positional arguments in format string, {}",
                placeholders.len(),
                arg_message
            ),
        )
        .to_compile_error()
        .into();
    }

    if !placeholders.is_empty() {
        // The parts of the format string with the placeholders replaced by arguments.
        let mut replaced_parts = Vec::new();

        let parts: Vec<&str> = placeholder_regex.split(&parsed_string).collect();
        let part_iter = parts.iter();

        let mut arg_iter = args.iter();
        let mut ph_iter = placeholders.iter();

        // Replace each occurrence of `{}` with their corresponding argument value.
        for part in part_iter {
            if !part.is_empty() {
                replaced_parts.push(quote! { logger.append(#part) });
            }

            if let Some(arg) = arg_iter.next() {
                // The number of placeholders was validated to be the same as
                // the number of arguments, so this should never panic.
                let placeholder = ph_iter.next().unwrap();

                match *placeholder {
                    "{}" => {
                        replaced_parts.push(quote! { logger.append(#arg) });
                    }
                    value if value.starts_with("{:.") => {
                        let precision =
                            if let Ok(precision) = value[3..value.len() - 1].parse::<u8>() {
                                precision
                            } else {
                                return Error::new_spanned(
                                    format_string,
                                    format!("invalid precision format: {}", value),
                                )
                                .to_compile_error()
                                .into();
                            };

                        replaced_parts.push(quote! {
                            logger.append_with_args(
                                #arg,
                                &[pinocchio_log::logger::Argument::Precision(#precision)]
                            )
                        });
                    }
                    value if value.starts_with("{:<.") || value.starts_with("{:>.") => {
                        let size = if let Ok(size) = value[4..value.len() - 1].parse::<usize>() {
                            size
                        } else {
                            return Error::new_spanned(
                                format_string,
                                format!("invalid truncate size format: {}", value),
                            )
                            .to_compile_error()
                            .into();
                        };

                        match value.chars().nth(2) {
                            Some('<') => {
                                replaced_parts.push(quote! {
                                    logger.append_with_args(
                                        #arg,
                                        &[pinocchio_log::logger::Argument::TruncateStart(#size)]
                                    )
                                });
                            }
                            Some('>') => {
                                replaced_parts.push(quote! {
                                    logger.append_with_args(
                                        #arg,
                                        &[pinocchio_log::logger::Argument::TruncateEnd(#size)]
                                    )
                                });
                            }
                            _ => {
                                // This should not happen since we already checked the format.
                                return Error::new_spanned(
                                    format_string,
                                    format!("invalid truncate format: {}", value),
                                )
                                .to_compile_error()
                                .into();
                            }
                        }
                    }
                    _ => {
                        return Error::new_spanned(
                            format_string,
                            format!("invalid placeholder: {}", placeholder),
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
        }

        // Generate the output string as a compile-time constant
        TokenStream::from(quote! {
            {
                let mut logger = pinocchio_log::logger::Logger::<#buffer_len>::default();
                #(#replaced_parts;)*
                logger.log();
            }
        })
    } else {
        TokenStream::from(quote! {pinocchio_log::logger::log_message(#format_string.as_bytes());})
    }
}

/// Attribute macro for instrumenting functions with compute unit logging.
///
/// This macro wraps the decorated function with additional logging statements
/// that print the function name and the number of compute units used before and after
/// the function execution.
///
/// # Usage
///
/// ```rust,ignore
/// #[compute_fn]
/// fn my_function() {
///     // Function body
/// }
/// ```
///
/// # Effects
///
/// - Adds a log message with the function name at the start of execution.
/// - Logs the number of compute units before and after the function execution.
/// - Adds a closing log message with the function name at the end of execution.
///
/// # Note on Compute Units Used by `compute_fn!`
///
/// ## Testing Results (as of 2024-09-01)
///
///  TOTAL COST OF LOGGING: 445 - 36 = 409
///  EXTRA COST INSIDE THE FUNCTION: 101
///
///  EMPTY_WITH_LOG where nothing happens inside the log.
///  TOTAL COMPUTE UNITS USED: 445
///  INNER LOG COST: 101
///
///  "Program EMPTY_WITH_LOG invoke [1]",
///  "Program log: Program log: process_instruction {{",
///  "Program consumption: 199762 units remaining",
///  "Program consumption: 199661 units remaining", // 199762 - 199661 = 101
///  "Program log: Program log: }} // process_instruction",
///  "Program EMPTY_WITH_LOG consumed 445 of 200000 compute units",
///  "Program EMPTY_WITH_LOG success"
///
///  EMPTY where nothing happens at all.
///  TOTAL COMPUTE UNITS USED: 36
///  
///  "Program EMPTY invoke [1]",
///  "Program EMPTY consumed 36 of 200000 compute units",
///  "Program EMPTY success"
///  
///  Total extra compute units used per `compute_fn!` call: 409 CU
///  For more details, see:
///  - https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/programs/bpf_loader/src/syscalls/logging.rs#L70
///  - https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/program-runtime/src/compute_budget.rs#L150
#[proc_macro_attribute]
pub fn compute_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let block = &input.block;

    input.block = syn::parse_quote!({
        ::pinocchio::msg!(concat!(stringify!(#fn_name), " {{"));
        ::pinocchio::log::sol_log_compute_units();

        let __result = (|| #block)();

        ::pinocchio::log::sol_log_compute_units();
        ::pinocchio::msg!(concat!("}} // ", stringify!(#fn_name)));

        __result
    });

    quote!(#input).into()
}
