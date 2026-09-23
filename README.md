# AWS Lambda Workload Identity & Least-Privilege IAM Architecture

## Overview

This project demonstrates how application identity and authorization change when an AWS workload moves from a local development environment to AWS Lambda.

A Rust application uses the AWS SDK to enumerate Amazon S3 buckets. The application was first executed locally using the developer's AWS credential context and was then adapted and deployed as an AWS Lambda function using a dedicated Lambda execution role.

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

The primary objectives of this project are to demonstrate:

* AWS workload identity
* Authentication versus authorization
* AWS SDK credential resolution
* IAM execution roles
* Least-privilege access
* AWS API authorization
* Lambda workload identity
* Secure credential handling
* Application failure-path testing
* Cloud security troubleshooting

Rust is used as the implementation language, but the primary focus of the project is cloud security architecture and IAM behavior.

---

## Architecture

### Local Execution

During local development, the application uses the AWS SDK credential provider mechanisms available in the developer environment.

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
        |
        v
ListBuckets Response
```

The application does not contain hard-coded AWS access keys.

The AWS SDK obtains an available credential context and uses those credentials to sign the request sent to AWS.

---

### Lambda Execution

After deployment, the workload no longer depends on the developer's local AWS credentials.

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
     |
     v
ListBuckets Response
```

This creates an important security boundary.

The application's code performs essentially the same AWS operation, but the identity under which the operation occurs has changed.

---

## Authentication vs. Authorization

One of the primary lessons demonstrated by this project is the distinction between authentication and authorization.

### Authentication

Authentication answers:

> Who is making this request?

Locally, the AWS SDK operates using the developer's available AWS credential context.

In Lambda, AWS provides the workload with temporary credentials associated with the Lambda execution role.

### Authorization

Authorization answers:

> What is this identity permitted to do?

The Lambda function had a valid AWS identity and could execute successfully, but its execution role initially lacked permission to enumerate S3 buckets.

The resulting application failure demonstrated that successful authentication does not imply successful authorization.

---

## IAM Least-Privilege Design

The application requires the ability to enumerate S3 buckets.

The required IAM action is:

```text
s3:ListAllMyBuckets
```

Rather than assigning broad S3 administrative or read permissions, the Lambda execution role was granted the specific action required by the workload.

Conceptually, the permission is:

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

The wildcard resource does not mean that the function has unrestricted access to every S3 operation. `ListAllMyBuckets` is an account-level S3 action and is not scoped to an individual bucket ARN.

No permissions to read, modify, upload, or delete S3 objects are required for the application.

---

## Application Flow

The application performs the following sequence:

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
        +------ Denied ------> Application Error Handling
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
Results Returned
```

---

## Security Failure Testing

The project intentionally tested failure conditions rather than validating only the successful execution path.

### Missing Credential Test

The local application was executed with its normal credential sources intentionally made unavailable.

Instead of relying solely on an unhandled AWS SDK error, application error handling was added to provide a controlled failure message.

This demonstrates the importance of designing cloud applications for authentication and service-access failures.

### Lambda Authorization Failure

The Lambda function was successfully built and deployed.

Initial invocation produced an application failure when the workload attempted to access S3.

Troubleshooting followed the execution path:

```text
Lambda Deployment
        |
        v
Successful Invocation
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
Lambda Execution Role Inspected
        |
        v
Required IAM Permission Identified
        |
        v
Least-Privilege Permission Added
        |
        v
Successful Invocation
```

This distinction is important operationally:

**Successful infrastructure deployment does not guarantee successful application execution.**

---

## Function Output

After the execution role was granted the required permission, the deployed Lambda function successfully returned an S3 bucket count and bucket list.

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

## Technologies

* Amazon Web Services (AWS)
* AWS Lambda
* AWS Identity and Access Management (IAM)
* Amazon S3
* AWS SDK for Rust
* Rust
* Cargo
* Cargo Lambda
* AWS CLI
* Linux / WSL

---

## Key Security Takeaways

### Identity belongs to the workload context

Application code does not inherently possess an AWS identity.

The identity used by the application depends on where and how the workload executes.

### Avoid embedded credentials

No AWS access keys are embedded in the application source code.

AWS-native workload identity should be used when the application executes within AWS.

### Authentication and authorization are separate controls

An authenticated Lambda workload can still be denied access to an AWS service when its execution role does not authorize the requested operation.

### Apply least privilege

Only the IAM capability required by the workload should be granted.

A simple bucket-enumeration function does not require broad S3 access.

### Test failure paths

Cloud architecture should account for credential failures, authorization failures, service failures, and other non-happy-path conditions.

### Deployment success is not application success

Infrastructure can deploy correctly while application dependencies, IAM permissions, or downstream service interactions still fail.

---

## Repository Structure

```text
aws-lambda-s3-workload-identity/
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

This project was adapted from an educational AWS SDK for Rust lab.

The original exercise demonstrated basic Amazon S3 bucket enumeration using the AWS SDK for Rust.

The project was extended to explore cloud security architecture concepts including:

* Lambda deployment
* AWS workload identity
* Lambda execution roles
* IAM authorization
* Least-privilege permission design
* Credential failure handling
* Authorization failure troubleshooting
* Local identity versus cloud workload identity

The purpose of this repository is to document and demonstrate the security architecture concepts explored through those extensions rather than to present the original educational example as independently authored code.

---

## Additional Documentation

Detailed documentation is provided in:

* `docs/architecture.md` — workload identity, trust boundaries, and execution flows
* `docs/iam-security.md` — IAM authorization and least-privilege analysis
* `docs/lessons-learned.md` — implementation, troubleshooting, and architectural lessons
