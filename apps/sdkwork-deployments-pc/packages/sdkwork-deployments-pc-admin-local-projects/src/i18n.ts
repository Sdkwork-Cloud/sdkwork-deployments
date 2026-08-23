import type { DeploymentsLocale } from "@sdkwork/deployments-pc-commons";

const enUs = {
  "page.eyebrow": "local projects",
  "page.title": "Local Projects",
  "page.description":
    "Manage modules and local runtime nodes through the Drive deploy sandbox. Physical host paths stay on the server.",
  "section.modules": "Modules",
  "section.nodes": "Local nodes",
  "section.browser": "Directory browser",
  "modules.empty": "No modules found under the deploy sandbox yet. Confirm the container cloned the workspace.",
  "modules.error": "Unable to load modules from the deploy sandbox.",
  "modules.refresh": "Refresh modules",
  "modules.open": "Open module",
  "nodes.open": "Browse node files",
  "browser.hint.module": "Browsing module {name}",
  "browser.hint.node": "Browsing files for node {name}",
  "browser.hint.root": "Browsing deploy sandbox root",
  "browser.close": "Close browser",
  "browser.missingPort": "Drive sandbox explorer is not configured for this Admin session.",
  "sandbox.missing": "Deploy sandbox is not available. Ensure Drive started with the deploy sandbox environment.",
  "node.development": "Docker Development",
  "node.development.description": "Local development container deploy tree",
  "node.test": "Docker Test",
  "node.test.description": "Local test container deploy tree",
  "node.production": "Docker Production",
  "node.production.description": "Local production container deploy tree",
  "node.host": "Local Host",
  "node.host.description": "Host-side deploy workspace view (same sandbox)",
  "kind.directory": "Directory",
  "kind.file": "File",
} as const;

const zhCn: Record<keyof typeof enUs, string> = {
  "page.eyebrow": "本地项目",
  "page.title": "本地项目管理",
  "page.description":
    "通过 Drive 部署沙箱管理模块与本地运行时节点。物理主机路径仅保留在服务端。",
  "section.modules": "模块列表",
  "section.nodes": "本地节点",
  "section.browser": "目录浏览器",
  "modules.empty": "部署沙箱下暂无模块。请确认容器已克隆工作空间仓库。",
  "modules.error": "无法从部署沙箱加载模块。",
  "modules.refresh": "刷新模块",
  "modules.open": "打开模块",
  "nodes.open": "浏览节点目录",
  "browser.hint.module": "正在浏览模块 {name}",
  "browser.hint.node": "正在浏览节点 {name} 的文件",
  "browser.hint.root": "正在浏览部署沙箱根目录",
  "browser.close": "关闭浏览器",
  "browser.missingPort": "当前 Admin 会话未配置 Drive 沙箱浏览器。",
  "sandbox.missing": "部署沙箱不可用。请确认 Drive 已启用部署沙箱环境变量。",
  "node.development": "Docker 开发环境",
  "node.development.description": "本地开发容器部署目录",
  "node.test": "Docker 测试环境",
  "node.test.description": "本地测试容器部署目录",
  "node.production": "Docker 生产环境",
  "node.production.description": "本地生产容器部署目录",
  "node.host": "本机 Host",
  "node.host.description": "本机部署工作区视图（同一沙箱）",
  "kind.directory": "目录",
  "kind.file": "文件",
};

export type LocalProjectsMessageKey = keyof typeof enUs;

export function translateLocalProjects(
  locale: DeploymentsLocale,
  key: LocalProjectsMessageKey,
  values: Record<string, string | number> = {},
): string {
  const catalog = locale === "zh-CN" ? zhCn : enUs;
  return Object.entries(values).reduce(
    (message, [name, value]) => message.replaceAll(`{${name}}`, String(value)),
    catalog[key],
  );
}
