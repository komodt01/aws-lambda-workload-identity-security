# AWS Lambda Workload Identity & Least-Privilege IAM Architecture

## Overview

This project demonstrates how application identity and authorization change when an AWS workload moves from a local development environment to AWS Lambda.

A Rust application uses the AWS SDK to enumerate Amazon S3 buckets. The application was first executed locally using the available developer AWS credential context and was then adapted and deployed as an AWS Lambda function using a dedicated Lambda execution role.

During deployment testing, the Lambda function successfully executed but initially failed when attempting to call the Amazon S3 API. Investigation identified that the Lambda execution role lacked the required S3 authorization.

Rather than assigning broad S3 permissions, the execution role was granted the specific IAM action required by the workload:

```text
s3:ListAllMyBuckets
```

The function was then successfully invoked and returned the available S3 bucket information.

The project demonstrates a fundamental cloud security principle:

> A workload having an authenticated AWS identity does not mean that identity is authorized to perform every AWS API operation.

---

## Security Objectives

The project demonstrates:

- AWS workload identity
- Authentication versus authorization
- AWS SDK credential resolution
- Lambda execution roles
- Least-privilege IAM
- AWS API authorization
- Secure credential handling
- Failure-path testing
- Cloud security troubleshooting

Rust is the implementation language, but the primary focus is cloud security architecture and IAM behavior.

---

## Architecture

### Local Execution

During local development, the application uses the AWS SDK credential mechanisms available in the developer environment.

```text
Developer Workstation
        |
        v
Rust Application
        |
        v
AWS SDK for Rust
        |
        v
Local AWS Credential Context
        |
        v
AWS IAM Authorization
        |
        v
Amazon S3 API
```

The application does not contain hard-coded AWS access keys.

The AWS SDK obtains an available credential context and uses those credentials to sign requests sent to AWS.

### Lambda Execution

After deployment, the workload no longer depends on the developer's local AWS credential context.

```text
AWS Lambda
     |
     v
Rust Application
     |
     v
AWS SDK for Rust
     |
     v
Lambda Execution Role
     |
     v
Temporary AWS Credentials
     |
     v
AWS IAM Authorization
     |
     v
Amazon S3 API
```

The application performs essentially the same AWS operation, but the identity under which that operation occurs has changed.

This creates an important security boundary between application code and workload identity.

---

## Authentication vs. Authorization

Authentication answers:

> Who or what is making this request?

Locally, the AWS SDK operates using the available developer credential context.

In Lambda, AWS provides temporary credentials associated with the function's execution role.

Authorization answers:

> What is this identity permitted to do?

The Lambda function had a valid AWS identity and could execute, but its execution role initially lacked permission to enumerate S3 buckets.

The resulting failure demonstrated that successful authentication does not imply successful authorization.

---

## IAM Least-Privilege Design

The application requires one S3 capability: enumerating the buckets available to the account.

The corresponding IAM action is:

```text
s3:ListAllMyBuckets
```

A policy representing that permission is:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "s3:ListAllMyBuckets",
      "Resource": "*"
    }
  ]
}
```

The `Resource: "*"` value does not grant unrestricted S3 access. `s3:ListAllMyBuckets` is an account-level action and is not scoped to an individual bucket or object ARN.

The workload does not require permissions such as:

```text
s3:GetObject
s3:PutObject
s3:DeleteObject
s3:CreateBucket
s3:DeleteBucket
s3:PutBucketPolicy
```

The security decision is therefore based on the capability the workload actually requires rather than attaching a broad S3 policy.

---

## Application and Authorization Flow

```text
Application Starts
        |
        v
AWS Configuration Loaded
        |
        v
AWS SDK S3 Client Created
        |
        v
ListBuckets API Request
        |
        v
AWS Authenticates Request
        |
        v
IAM Evaluates Authorization
        |
        +------ Denied ------> SDK Error / Lambda Failure
        |
      Allowed
        |
        v
S3 Returns Bucket Metadata
        |
        v
Application Processes Response
        |
        v
JSON Response Returned
```

This separates several concerns that can otherwise be confused during troubleshooting:

```text
Application Code
      ↓
AWS SDK
      ↓
Credential Source
      ↓
AWS Principal
      ↓
IAM Authorization
      ↓
AWS API
      ↓
Response or Failure
```

---

## Security Failure Testing

The project included failure-path testing rather than validating only successful execution.

### Missing Credential Test

The local application was tested with its normal AWS credential sources intentionally made unavailable.

The test demonstrated how the application behaves when the AWS SDK cannot obtain a usable credential context.

In the current implementation, AWS SDK errors are propagated rather than handled through custom application-specific error logic.

This demonstrates why credential and service-access failures should be considered explicitly when designing cloud workloads.

### Lambda Authorization Failure

The Lambda function was successfully built and deployed, but its initial S3 request failed because the execution role did not authorize the required S3 action.

Troubleshooting followed the request path:

```text
Lambda Deployment
        |
        v
Function Invocation
        |
        v
Application Executes
        |
        v
S3 API Request
        |
        v
Authorization Failure
        |
        v
Execution Role Inspected
        |
        v
Required IAM Action Identified
        |
        v
Least-Privilege Permission Added
        |
        v
Successful Invocation
```

This demonstrates an important operational distinction:

> Successful infrastructure deployment does not guarantee successful application execution.

---

## Function Output

After the execution role was granted the required permission, the Lambda function returned an S3 bucket count and bucket list.

Example sanitized output:

```json
{
  "bucket_count": 1,
  "buckets": [
    "example-bucket"
  ]
}
```

Real AWS account identifiers and resource names are intentionally excluded from this repository.

---

## Security Architecture Principles

### Workload Identity Over Embedded Credentials

The application does not contain long-lived AWS access keys.

When deployed to Lambda, the workload uses the AWS identity associated with its execution role and temporary credentials provided through the AWS runtime environment.

### Least Privilege

IAM authorization should reflect the capabilities a workload actually requires.

For this workload:

```text
Required capability:
Enumerate S3 buckets

Required IAM action:
s3:ListAllMyBuckets

Not required:
s3:*
```

### Authentication and Authorization Are Separate

A workload may possess a valid AWS identity while still being denied access to a service operation.

Identity establishes the principal. IAM determines what that principal is authorized to do.

### Execution Context Determines Identity

The same application code can execute under different AWS identities depending on its environment.

Local execution uses the available developer credential context, while Lambda execution uses the function's execution role.

### Failure Paths Are Part of Architecture

Credential failures, authorization failures, downstream service failures, and application failures should be considered alongside the successful execution path.

### Deployment Success Is Not Runtime Success

Successful compilation and deployment do not prove that IAM permissions, external dependencies, or application behavior are correct.

Runtime validation is still required.

---

## Production Considerations

This project focuses on workload identity and IAM behavior rather than implementing a complete production security architecture.

A production implementation could additionally include:

- Infrastructure as Code for execution roles and IAM policies
- IAM policy review and approval workflows
- CloudTrail logging and monitoring
- Role ownership and lifecycle governance
- Permission boundaries where appropriate
- AWS Organizations service control policies
- Continuous IAM analysis and configuration-drift detection
- Application-specific error handling and observability

These are architectural extensions rather than controls implemented by this repository.

---

## Technologies

- Amazon Web Services (AWS)
- AWS Lambda
- AWS Identity and Access Management (IAM)
- Amazon S3
- AWS SDK for Rust
- Rust
- Cargo
- Cargo Lambda
- AWS CLI
- Linux / WSL

---

## Repository Structure

```text
aws-lambda-workload-identity-security/
├── README.md
├── Cargo.toml
├── src/
│   └── main.rs
├── docs/
│   ├── architecture.md
│   ├── iam-security.md
│   └── lessons-learned.md
└── .gitignore
```

---

## Project Origin

This project was adapted from an educational AWS SDK for Rust exercise that demonstrated basic Amazon S3 bucket enumeration.

The implementation was extended to explore cloud security architecture concepts including:

- Lambda deployment
- AWS workload identity
- Lambda execution roles
- IAM authorization
- Least-privilege permission design
- Credential failure testing
- Authorization failure troubleshooting
- Local identity versus cloud workload identity

The purpose of this repository is to document and demonstrate the security architecture concepts explored through those extensions rather than present the original educational example as independently authored code.

---

## Additional Documentation

Additional analysis is available in:

- `docs/architecture.md` — workload identity, trust boundaries, and execution flows
- `docs/iam-security.md` — IAM authorization and least-privilege analysis
- `docs/lessons-learned.md` — implementation, troubleshooting, and architectural lessons

---

## Key Takeaway

The important security lesson is not the S3 bucket-listing operation itself.

The project demonstrates how application code, workload identity, temporary credentials, IAM authorization, and AWS service APIs interact as one system.

```text
AWS Lambda
     |
     v
Application
     |
     v
AWS SDK
     |
     v
Execution Role
     |
     v
Temporary Credentials
     |
     v
IAM Authorization
     |
     v
Amazon S3
```

Secure cloud access depends on both **which identity performs an operation** and **exactly what that identity is authorized to do**.
