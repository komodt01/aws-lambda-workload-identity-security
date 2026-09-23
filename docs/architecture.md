# AWS Lambda Workload Identity Architecture

## Purpose

This document describes the security architecture of the AWS Lambda Workload Identity & Least-Privilege IAM project.

The project demonstrates how the security context of an application changes when the same AWS SDK operation moves from a developer workstation to an AWS-managed compute environment.

The central architectural principle is:

> Application code does not determine its AWS identity. The execution environment and associated credential mechanism determine the identity under which AWS API requests are made.

---

## Architecture Overview

The application performs a simple operation:

1. Load AWS configuration.
2. Create an Amazon S3 client using the AWS SDK.
3. Call the S3 `ListBuckets` API.
4. Process the response.
5. Return the available bucket names.

The application was executed in two environments:

* Local Linux/WSL development environment
* AWS Lambda

Although the application operation remained essentially the same, the identity and trust model changed significantly between the two environments.

---

## Local Execution Architecture

```text
Developer
    |
    v
Linux / WSL Environment
    |
    v
Rust Application
    |
    v
AWS SDK for Rust
    |
    v
AWS Credential Provider
    |
    v
Developer AWS Identity
    |
    v
Signed AWS API Request
    |
    v
AWS IAM Authorization
    |
    v
Amazon S3
```

### Execution Flow

The application initializes AWS configuration:

```rust
let config = aws_config::load_from_env().await;
```

An S3 client is then created:

```rust
let client = aws_sdk_s3::Client::new(&config);
```

The client sends the S3 request:

```rust
client.list_buckets().send().await
```

The AWS SDK signs the API request using credentials available through the local AWS credential context.

AWS then evaluates whether the resulting identity is authorized to perform the requested S3 action.

---

## Lambda Execution Architecture

After deployment, the security context changes.

```text
AWS Lambda Service
        |
        v
Lambda Function
        |
        v
Rust Runtime
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
Signed AWS API Request
        |
        v
AWS IAM Authorization
        |
        v
Amazon S3
```

The application no longer depends on credentials stored in the developer environment.

Instead, AWS provides temporary credentials associated with the Lambda execution role.

The SDK can therefore continue using its normal credential resolution behavior without requiring credentials to be embedded in application source code.

---

## Identity Transition

The deployment creates an important identity transition.

### Local

```text
Application
    |
    v
Developer Credential Context
    |
    v
Developer IAM Identity
```

### Lambda

```text
Application
    |
    v
Lambda Runtime Environment
    |
    v
Lambda Execution Role
    |
    v
Temporary Workload Credentials
```

The application logic can remain substantially unchanged while the identity changes completely.

This is an important cloud architecture pattern because it allows application code to remain separated from long-lived credentials.

---

## Trust Boundaries

Several trust boundaries exist within the architecture.

### Boundary 1 — Developer Environment to AWS

During local execution, AWS credentials available to the developer environment are used to authenticate API requests.

Security considerations include:

* protection of local AWS credentials
* credential scope
* profile configuration
* credential expiration
* developer privilege level
* avoidance of credentials in source control

---

### Boundary 2 — Lambda Runtime to AWS IAM

In AWS Lambda, the execution role establishes the workload's AWS identity.

The Lambda service assumes the execution role and makes temporary credentials available to the function runtime.

The application should not require embedded long-lived access keys.

---

### Boundary 3 — IAM to Amazon S3

Possession of a valid AWS identity does not automatically authorize the S3 operation.

IAM evaluates whether the principal is authorized to perform:

```text
s3:ListAllMyBuckets
```

If authorization is absent, S3 rejects the request.

---

## Authentication and Authorization Flow

```text
Lambda Function
      |
      v
Obtains Workload Credentials
      |
      v
AWS Request Signed
      |
      v
AWS Authenticates Principal
      |
      v
IAM Evaluates Requested Action
      |
      +-----------------------+
      |                       |
    Allow                    Deny
      |                       |
      v                       v
S3 Processes Request     Request Rejected
      |
      v
Response Returned
```

This illustrates why authentication and authorization must be treated as separate architectural controls.

---

## Failure Path

The Lambda function initially reached AWS successfully but could not complete its S3 operation.

```text
Lambda Starts
     |
     v
Execution Role Available
     |
     v
AWS SDK Creates S3 Client
     |
     v
ListBuckets Request
     |
     v
IAM Authorization Evaluation
     |
     v
DENIED
     |
     v
Application Failure
```

The failure was not caused by an inability to deploy Lambda or establish an AWS identity.

The workload was authenticated but insufficiently authorized.

---

## Remediated Path

The required permission was identified and added to the execution role:

```text
s3:ListAllMyBuckets
```

The resulting flow became:

```text
Lambda Starts
     |
     v
Execution Role Available
     |
     v
AWS SDK Creates S3 Client
     |
     v
ListBuckets Request
     |
     v
IAM Authorization Evaluation
     |
     v
ALLOWED
     |
     v
S3 Returns Bucket Metadata
     |
     v
Lambda Returns JSON Response
```

---

## Security Design Principles

### Workload Identity Over Embedded Credentials

Applications running on AWS should use AWS-native workload identity mechanisms when possible rather than embedding long-lived credentials.

### Least Privilege

The workload should receive only the actions required to perform its business function.

### Explicit Trust Boundaries

Identity transitions between developer environments, cloud runtimes, IAM, and AWS services should be understood and documented.

### Failure-Aware Architecture

Authorization failure, missing credentials, unavailable services, and other failure conditions should be considered part of the application architecture.

### Separation of Code and Authorization

Application code expresses the API operation it wants to perform.

IAM determines whether the workload is permitted to perform it.

These are separate architectural concerns.

---

## Architecture Takeaway

The most important architectural lesson from this project is not the S3 operation itself.

It is that the same application can behave differently when moved between environments because its identity and authorization context change.

Understanding that relationship is fundamental to designing secure cloud workloads.
