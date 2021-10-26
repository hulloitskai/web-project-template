import React, { useMemo } from "react";
import type { NextPage } from "next";

import { gql } from "@apollo/client";
import { useHomePageQuery } from "apollo";

import { VStack } from "@chakra-ui/react";
import { Skeleton } from "@chakra-ui/react";
import { Text } from "@chakra-ui/react";
import { chakra } from "@chakra-ui/react";

gql`
  query HomePage {
    buildInfo {
      timestamp
      version
    }
  }
`;

const HomePage: NextPage = () => {
  const { data } = useHomePageQuery();
  const dataText = useMemo(() => JSON.stringify(data, undefined, 2), [data]);
  return (
    <VStack spacing={4} my={24}>
      <Text fontSize="3xl" fontWeight="extrabold">
        HELLO WORLD
      </Text>
      <Skeleton isLoaded={!!data} minH={32}>
        <chakra.pre>{dataText}</chakra.pre>
      </Skeleton>
    </VStack>
  );
};

export default HomePage;
