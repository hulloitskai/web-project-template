import React from "react";
import type { NextPage } from "next";

import { Text } from "@chakra-ui/react";

const HomePage: NextPage = () => {
  return (
    <Text fontSize="3xl" fontWeight="bold">
      Hello World!
    </Text>
  );
};

export default HomePage;
