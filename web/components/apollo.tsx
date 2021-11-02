import { FC, ReactNode, useCallback, useContext, useMemo } from "react";
import { isEqual } from "lodash";
import merge from "deepmerge";

import { useToast } from "components/toast";

import { ApolloClient as Client } from "@apollo/client";
import { ApolloProvider as Provider } from "@apollo/client";
import { ApolloError, ServerError } from "@apollo/client";
import { getApolloContext } from "@apollo/client";

import { HttpLink } from "@apollo/client";
import { RetryLink } from "@apollo/client/link/retry";
import { SentryLink } from "apollo-link-sentry";
import { WebSocketLink as WsLink } from "@apollo/client/link/ws";
import { split as splitLinks } from "@apollo/client";
import { from as mergeLinks } from "@apollo/client";
import type { ApolloLink } from "@apollo/client";

import { getMainDefinition } from "@apollo/client/utilities";
import { InMemoryCache, NormalizedCacheObject } from "@apollo/client";
import { TypedTypePolicies as TypePolicies } from "apollo/helpers";

import { TEMPLATE_API_PUBLIC_URL, TEMPLATE_API_URL } from "consts";

const typePolicies: TypePolicies = {};

const createTerminatingLink = (): ApolloLink => {
  const httpLink = new HttpLink({
    uri:
      typeof window !== "undefined"
        ? `${TEMPLATE_API_PUBLIC_URL}/graphql`
        : `${TEMPLATE_API_URL}/graphql`,
  });
  if (typeof window === "undefined") {
    return httpLink;
  }

  const wsLink = new WsLink({
    uri: (() => {
      const { protocol, host, pathname } = new URL(TEMPLATE_API_PUBLIC_URL);
      const path = pathname.endsWith("/") ? pathname.slice(0, -1) : pathname;
      switch (protocol) {
        case "http:":
          return `ws://${host}${path}/graphql`;
        case "https:":
          return `wss://${host}${path}/graphql`;
        default:
          throw new Error("Unknown protocol.");
      }
    })(),
    options: {
      reconnect: true,
    },
  });
  return splitLinks(
    ({ query }) => {
      const definition = getMainDefinition(query);
      return (
        definition.kind === "OperationDefinition" &&
        definition.operation === "subscription"
      );
    },
    wsLink,
    httpLink,
  );
};

const createApolloClient = (): Client<NormalizedCacheObject> => {
  return new Client({
    ssrMode: typeof window === "undefined",
    link: mergeLinks([
      new RetryLink(),
      new SentryLink(),
      createTerminatingLink(),
    ]),
    cache: new InMemoryCache({ typePolicies }),
  });
};

export interface ApolloProviderProps {
  initialState?: NormalizedCacheObject;
  children: ReactNode | ReactNode[] | null;
}

export const ApolloProvider: FC<ApolloProviderProps> = ({
  initialState,
  children,
}) => {
  const context = useContext(getApolloContext());
  const client = useMemo(
    () => {
      if (context.client) {
        return context.client;
      }
      return initializeApolloClient(initialState);
    },
    [], // eslint-disable-line react-hooks/exhaustive-deps
  );
  return <Provider client={client}>{children}</Provider>;
};

let globalApolloClient: Client<NormalizedCacheObject> | undefined;

export const initializeApolloClient = (
  initialState?: NormalizedCacheObject,
): Client<NormalizedCacheObject> => {
  const client = globalApolloClient ?? createApolloClient();

  // Hydrate cache from initial state.
  if (initialState) {
    // Get existing cache, loaded during client side data fetching.
    const cache = client.extract();

    // Merge the existing cache with initial state.
    const state = merge(initialState, cache, {
      // combine arrays using object equality (like in sets)
      arrayMerge: (destinationArray, sourceArray) => [
        ...sourceArray,
        ...destinationArray.filter(d => sourceArray.every(s => !isEqual(d, s))),
      ],
    });

    // Restore the cache with the merged data.
    client.restore(state);
  }

  // For SSG and SSR always create a new client.
  if (typeof window === "undefined") {
    return client;
  }

  // For browser, use a global client.
  if (!globalApolloClient) {
    globalApolloClient = client;
  }

  return globalApolloClient;
};

export const useHandleQueryError = (
  title?: string,
): ((error: ApolloError) => void) => {
  const toast = useToast();
  return useCallback(
    error => {
      const description = formatApolloError(error);
      toast({
        status: "error",
        title,
        description,
      });
    },
    [title, toast],
  );
};

export const formatApolloError = (error: ApolloError): string => {
  const { graphQLErrors, networkError, message } = error;
  if (graphQLErrors) {
    const [firstError] = graphQLErrors;
    if (firstError) {
      return `Error: ${firstError.message}`;
    }
  }
  if (networkError) {
    if ((networkError as ServerError | undefined)?.statusCode === 500) {
      return "An internal server error occurred.";
    }
  }
  return message;
};
