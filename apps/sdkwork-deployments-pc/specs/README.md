# SDKWork Deployments PC Contract

The browser root composes an app-console publishing surface and a lazy backend-admin operations surface. One TokenManager is shared by IAM, Deploy App SDK, Drive App SDK, and Deploy Backend SDK clients. Drive owns byte upload and storage lifecycle; Deploy owns immutable artifact, release, deployment, and runtime-assignment business state.

