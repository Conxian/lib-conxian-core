# AWS Nitro Enclave Bootstrap — Admin Actions Required

## Context
User `botshelo` (account `692112933743`) has read-only EC2 + security-group + IAM-role permissions.
The following resources are already provisioned and ready:
- Security group: `sg-074fed552d15a1677` (SSH 22 + enclave 50051)
- IAM role: `conxian-nitro-enclave-role` (SSM managed instance core attached)
- GitHub secrets: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`

## Action 1: Attach Nitro Bootstrap Policy to botshelo

Apply this inline policy to IAM user `botshelo` via AWS Console or CLI:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "EC2Permissions",
      "Effect": "Allow",
      "Action": [
        "ec2:RunInstances",
        "ec2:CreateKeyPair"
      ],
      "Resource": "*"
    },
    {
      "Sid": "IAMInstanceProfilePermission",
      "Effect": "Allow",
      "Action": "iam:CreateInstanceProfile",
      "Resource": "*"
    },
    {
      "Sid": "IAMPassRolePermission",
      "Effect": "Allow",
      "Action": "iam:PassRole",
      "Resource": "arn:aws:iam::692112933743:role/conxian-nitro-enclave-role",
      "Condition": {
        "StringEquals": {
          "iam:PassedToService": "ec2.amazonaws.com"
        }
      }
    }
  ]
}
```

CLI equivalent:
```bash
aws iam put-user-policy \
  --user-name botshelo \
  --policy-name conxian-nitro-bootstrap \
  --policy-document file://docs/aws/nitro-bootstrap-policy.json
```

## Action 2 (Optional): GitHub OIDC Trust

For long-term CI-driven provisioning without static credentials:

```bash
# One-time OIDC provider setup
THUMBPRINT=$(openssl s_client -servername token.actions.githubusercontent.com \
  -showcerts -connect token.actions.githubusercontent.com:443 </dev/null 2>/dev/null \
  | openssl x509 -noout -fingerprint -sha1 | sed 's/.*=//;s/://g')

aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --client-id-list sts.amazonaws.com \
  --thumbprint-list "$THUMBPRINT"

# Create CI role
aws iam create-role \
  --role-name github-actions-nitro-provisioner \
  --assume-role-policy-document '{
    "Version": "2012-10-17",
    "Statement": [{
      "Effect": "Allow",
      "Principal": {"Federated": "arn:aws:iam::692112933743:oidc-provider/token.actions.githubusercontent.com"},
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:sub": "repo:Conxian/lib-conxian-core:ref:refs/heads/main"
        }
      }
    }]
  }'

# Attach full EC2 provisioning policy
aws iam attach-role-policy \
  --role-name github-actions-nitro-provisioner \
  --policy-arn arn:aws:iam::aws:policy/AmazonEC2FullAccess

aws iam attach-role-policy \
  --role-name github-actions-nitro-provisioner \
  --policy-arn arn:aws:iam::aws:policy/IAMFullAccess
```

Then update `.github/workflows/nitro-enclave-ci.yml` to replace:
```yaml
role-to-assume: arn:aws:iam::692112933743:role/github-actions-nitro-provisioner
```

## After Admin Actions

Once Action 1 is complete, run the CI workflow manually:
1. Go to https://github.com/Conxian/lib-conxian-core/actions/workflows/nitro-enclave-ci.yml
2. Click "Run workflow"
3. Set `provision_aws` = true
4. Click "Run workflow"

This will:
1. Build and test the enclave adapter (28 tests)
2. Run all 6 POC scenarios
3. Build the Nitro EIF Docker image
4. Launch an m5.xlarge spot instance (~$0.06/hr)
5. Deploy and test the enclave signing server
6. Auto-teardown the instance (4h TTL + always() cleanup)

Estimated POC cost: <$0.50 per run.
