import { describe, expect, it } from "vitest";

import {
  DEPLOY_LOCAL_SANDBOX_ID,
  DEPLOY_SPACE_MODULE_PARENT,
  LOCAL_DEPLOY_NODES,
} from "../packages/sdkwork-deployments-pc-admin-local-projects/src/constants.ts";

describe("local projects admin contract", () => {
  it("uses the stable deploy sandbox id", () => {
    expect(DEPLOY_LOCAL_SANDBOX_ID).toBe("deploy.local.opt_deploy");
    expect(DEPLOY_SPACE_MODULE_PARENT).toBe("sdkwork-space");
  });

  it("exposes local docker and host nodes", () => {
    expect(LOCAL_DEPLOY_NODES.map((node) => node.id)).toEqual([
      "local.docker.development",
      "local.docker.test",
      "local.docker.production",
      "local.host",
    ]);
  });
});
