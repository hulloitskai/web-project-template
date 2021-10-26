const { TEMPLATE_API_URL } = process.env;
const { AUTH0_DOMAIN, AUTH0_CLIENT_ID } = process.env;
const { GCP_API_KEY } = process.env;

module.exports = {
  productionBrowserSourceMaps: true,
  rewrites: async () => {
    if (!TEMPLATE_API_URL) {
      console.info("Missing API server URL; proxying is disabled.");
      return [];
    }
    return [
      {
        source: "/api/:slug*",
        destination: `${TEMPLATE_API_URL}/:slug*`,
      },
    ];
  },
  headers: async () => [
    {
      source: "/fonts/:font",
      headers: [
        {
          key: "Access-Control-Allow-Origin",
          value: "*",
        },
      ],
    },
  ],
  publicRuntimeConfig: {
    AUTH0_DOMAIN,
    AUTH0_CLIENT_ID,
    GCP_API_KEY,
  },
  serverRuntimeConfig: {
    AUTH0_DOMAIN,
    AUTH0_CLIENT_ID,
    GCP_API_KEY,
    TEMPLATE_API_URL,
  },
};
