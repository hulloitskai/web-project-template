import getConfig from "next/config";

const { publicRuntimeConfig, serverRuntimeConfig } = getConfig();

export const { TEMPLATE_API_URL } = {
  ...publicRuntimeConfig,
  ...serverRuntimeConfig,
} as any;
