import type { Children } from "@types";
import { Provider } from "effector-react";
import { fork } from "effector";
import React from "react";

export const AppProvider: React.FC<Children> = ({ children }) => {
  const scope = fork({
    values: []
  });
  return <React.StrictMode>
    <Provider value={scope}>
      {children}
    </Provider>
  </React.StrictMode>;
};