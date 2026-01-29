# Security Policy

1. [Reporting security problems](#reporting)
2. [Incident Response Process](#process)
3. [Security Bug Bounties](#bounty)

<a name="reporting"></a>
## Reporting security problems in Pinocchio

**DO NOT CREATE A GITHUB ISSUE** to report a security problem.

Instead please use this [Report a Vulnerability](https://github.com/anza-xyz/pinocchio/security/advisories/new) link.
Provide a helpful title, detailed description of the vulnerability and an exploit
proof-of-concept. Speculative submissions without proof-of-concept will be closed
with no further consideration.

Please refer to the [Agave security policy](https://github.com/anza-xyz/agave/security/policy)
for vulnerabilities regarding Agave.

If you haven't done so already, please **enable two-factor auth** in your GitHub account.

Expect a response as fast as possible in the advisory, typically within 72 hours.

--

If you do not receive a response in the advisory, send an email to
security@anza.xyz with the full URL of the advisory you have created. DO NOT
include attachments or provide detail sufficient for exploitation regarding the
security issue in this email. **Only provide such details in the advisory**.

If you do not receive a response from security@anza.xyz please followup with
the team directly. You can do this in the `#core-technology` channel of the
[Solana Tech discord server](https://solana.com/discord), by pinging the `Anza`
role in the channel and referencing the fact that you submitted a security problem.

<a name="process"></a>
## Incident Response Process

In case an incident is discovered or reported, the following process will be
followed to contain, respond and remediate:

### 1. Accept the new report
In response a newly reported security problem, a member of the
`anza-xyz/admins` group will accept the report to turn it into a draft
advisory.  The `anza-xyz/security-incident-response` group should be added to
the draft security advisory, and create a private fork of the repository (grey
button towards the bottom of the page) if necessary.

If the advisory is the result of an audit finding, follow the same process as above but add the auditor's github user(s) and begin the title with "[Audit]".

If the report is out of scope, a member of the `anza-xyz/admins` group will
comment as such and then close the report.

### 2. Triage
Within the draft security advisory, discuss and determine the severity of the issue. If necessary, members of the anza-xyz/security-incident-response group may add other github users to the advisory to assist.
If it is determined that this is not a critical network issue then the advisory should be closed and if more follow-up is required a normal public GitHub issue should be created.

### 3. Prepare Fixes
For the affected packages, prepare a fix for the issue and push them to the private repository associated with the draft security advisory.
There is no CI available in the private repository so you must build from source and manually verify fixes.
Code review from the reporter is ideal, as well as from multiple members of the core development team.

### 4. Notify Security Group Validators
Once an ETA is available for the fix, a member of the anza-xyz/security-incident-response group should notify the validators so they can prepare for an update using the "Solana Red Alert" notification system.
The teams are all over the world and it's critical to provide actionable information at the right time. Don't be the person that wakes everybody up at 2am when a fix won't be available for hours.

### 5. Ship the patch
Once the fix is accepted it may be distributed directly to validators as a patch, depending on the vulnerability.

### 6. Public Disclosure and Release
Once the fix has been deployed to the security group validators, the patches from the security advisory may be merged into the main source repository. A new official release for each affected branch should be shipped and all validators requested to upgrade as quickly as possible.

### 7. Security Advisory Bounty Accounting and Cleanup
If this issue is [eligible](#eligibility) for a bounty, prefix the title of the
security advisory with one of the following, depending on the severity:
- [Bounty Category: Critical: Loss of Funds]
- [Bounty Category: Critical: Consensus / Safety Violations]
- [Bounty Category: Critical: Liveness / Loss of Availability]
- [Bounty Category: Critical: DoS Attacks]
- [Bounty Category: Supply Chain Attacks]
- [Bounty Category: RPC]

Confirm with the reporter that they agree with the severity assessment, and discuss as required to reach a conclusion.

We currently do not use the GitHub workflow to publish security advisories. Once the issue and fix have been disclosed, and a bounty category is assessed if appropriate, the GitHub security advisory is no longer needed and can be closed.

<a name="bounty"></a>
## Security Bug Bounties
At its sole discretion, the Solana Foundation may offer a bounty for
[valid reports](#reporting) of critical vulnerabilities.

Please see the [Agave Security Bug
Bounties](https://github.com/anza-xyz/agave/security/policy#security-bug-bounties)
for details on classes of bugs and payment amounts.