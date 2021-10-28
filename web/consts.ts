import getConfig from "next/config";

const { publicRuntimeConfig, serverRuntimeConfig } = getConfig();

export const { TEMPLATE_API_URL, TEMPLATE_API_PUBLIC_URL } = {
  ...publicRuntimeConfig,
  ...serverRuntimeConfig,
} as any;
