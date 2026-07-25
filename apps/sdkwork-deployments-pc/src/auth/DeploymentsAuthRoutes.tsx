import { SdkworkAuthPage, type SdkworkAuthController, type SdkworkAuthHeaderSlotProps } from "@sdkwork/auth-pc-react"; import { PackageCheck } from "lucide-react";
export function DeploymentsAuthRoutes({ controller }: { controller: SdkworkAuthController }) { return <SdkworkAuthPage appearance={{ slots: { Header } }} basePath="/auth" controller={controller} homePath="/console" />; }
function Header({ description, title }: SdkworkAuthHeaderSlotProps) { return <header className="auth-header"><div className="auth-brand"><PackageCheck size={18} /><strong>SDKWork Deployments</strong></div><h1>{title}</h1><p>{description}</p></header>; }

