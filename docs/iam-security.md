# IAM Security and Least-Privilege Analysis

## Purpose

This document examines the IAM security model used by the AWS Lambda Workload Identity & Least-Privilege IAM Lab.

The project demonstrates how AWS authentication, workload identity, IAM authorization, and least-privilege permissions interact when an application accesses Amazon S3.

---

## Security Requirement

The Lambda function performs one S3 operation:

```text
ListBuckets
```

The corresponding IAM action is:

```text
s3:ListAllMyBuckets
```

The workload does not require the ability to:

* retrieve S3 objects
* upload S3 objects
* delete S3 objects
* modify bucket policies
* change bucket configuration
* create buckets
* delete buckets

The IAM design should therefore not provide those capabilities.

---

## Lambda Execution Role

AWS Lambda functions use an execution role to interact with AWS services.

Conceptually:

```text
Lambda Function
      |
      v
Execution Role
      |
      v
Temporary Credentials
      |
      v
AWS API
```

The execution role represents the workload's AWS identity.

This eliminates the need to store long-lived AWS access keys inside the application.

---

## Initial Authorization State

The Lambda function was successfully deployed with an execution role.

However, successful deployment did not provide the role with permission to enumerate S3 buckets.

The resulting flow was:

```text
Lambda
   |
   v
Valid Execution Role
   |
   v
Authenticated AWS Request
   |
   v
ListBuckets
   |
   v
IAM Evaluation
   |
   v
DENY
```

This demonstrates an important IAM concept:

> An AWS principal can be successfully authenticated while still being unauthorized to perform a requested action.

---

## Required Permission

The required IAM action was:

```text
s3:ListAllMyBuckets
```

A least-privilege policy for this specific operation is conceptually:

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

---

## Understanding Resource "*"

At first glance, the following may appear overly broad:

```json
"Resource": "*"
```

However, IAM least privilege must be evaluated across the entire statement, not by examining the `Resource` element alone.

The action is limited to:

```text
s3:ListAllMyBuckets
```

This operation enumerates buckets at the account level and is not authorized against an individual S3 bucket or object ARN.

The policy does not grant broad S3 access.

For example, it does not grant:

```text
s3:GetObject
s3:PutObject
s3:DeleteObject
s3:CreateBucket
s3:DeleteBucket
s3:PutBucketPolicy
```

Therefore:

```text
Action: s3:ListAllMyBuckets
Resource: *
```

is fundamentally different from:

```text
Action: s3:*
Resource: *
```

The latter would represent dramatically broader privilege.

---

## Least-Privilege Decision

A broad managed policy was not required for the workload.

Instead, the permission was constrained to the API capability necessary for the application.

```text
Required Function:
Enumerate S3 buckets

Required IAM Action:
s3:ListAllMyBuckets

Unnecessary:
s3:*
```

This reduces the potential impact if the workload is compromised.

---

## Credential Security

No long-lived AWS credentials are embedded in the Lambda source code.

The preferred model is:

```text
AWS Lambda
    |
    v
Execution Role
    |
    v
Temporary Credentials
    |
    v
AWS SDK
```

rather than:

```text
Source Code
    |
    v
Hard-Coded Access Key
    |
    v
AWS API
```

Hard-coded credentials create several risks:

* accidental exposure through source control
* difficult credential rotation
* credential reuse
* excessive credential lifetime
* unclear ownership
* increased blast radius

Workload identity substantially reduces these risks.

---

## Local Credentials vs. Workload Credentials

The project demonstrated two credential contexts.

### Developer Context

During local execution:

```text
Developer
   |
   v
Local AWS Credential Context
   |
   v
AWS SDK
```

The application's permissions are determined by the AWS identity associated with that credential context.

### Lambda Context

During Lambda execution:

```text
Lambda
   |
   v
Execution Role
   |
   v
Temporary Credentials
   |
   v
AWS SDK
```

The application's permissions are determined by the Lambda execution role.

This means successful local testing does not prove that the deployed workload has the correct permissions.

---

## Security Testing

The project intentionally tested multiple security states.

### Valid Local Credentials

The application successfully authenticated and listed S3 buckets.

### Missing Local Credentials

Credential sources were intentionally made unavailable for a test execution.

The application was modified to handle the failure more gracefully.

### Valid Lambda Identity, Insufficient Authorization

The Lambda function had an execution role but lacked S3 enumeration permission.

The application failed when it attempted the S3 API operation.

### Valid Lambda Identity, Correct Authorization

After the specific IAM permission was granted, the Lambda successfully completed the request.

These tests demonstrate multiple security states:

```text
No Identity
    |
    v
Authentication Failure

Valid Identity + Insufficient Permission
    |
    v
Authorization Failure

Valid Identity + Required Permission
    |
    v
Successful Operation
```

---

## IAM Troubleshooting Method

The authorization issue was approached by tracing the request path rather than immediately granting broad permissions.

```text
1. Confirm Lambda deployment
          |
          v
2. Invoke function
          |
          v
3. Observe application failure
          |
          v
4. Identify Lambda execution role
          |
          v
5. Determine required AWS API action
          |
          v
6. Add specific IAM permission
          |
          v
7. Invoke again
          |
          v
8. Confirm successful operation
```

This approach avoids treating broad administrative access as a troubleshooting mechanism.

---

## Production Considerations

A production implementation would extend this design with additional controls.

### Infrastructure as Code

The execution role and IAM policies should be defined through controlled infrastructure-as-code rather than manual configuration.

### Policy Review

IAM policies should undergo code review and security review before deployment.

### CloudTrail Monitoring

AWS API activity should be captured through CloudTrail to support auditability and investigation.

### Role Governance

Execution roles should have clear ownership, naming standards, lifecycle management, and periodic access review.

### Permission Boundaries

Organizations may use permission boundaries or other governance controls to limit the maximum permissions that workload roles can receive.

### Organizational Guardrails

AWS Organizations service control policies may provide additional boundaries on what workload identities can perform.

### Continuous Validation

IAM configuration should be continuously evaluated for excessive permissions and configuration drift.

---

## Security Takeaway

IAM is not simply a configuration step required to make an application work.

It is an architectural enforcement layer between workloads and cloud resources.

The objective is not:

> Give the application enough access so the error disappears.

The objective is:

> Identify the exact capability the workload requires and authorize that capability without unnecessarily expanding the workload's blast radius.
