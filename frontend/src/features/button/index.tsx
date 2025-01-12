import { TonConnectUIProvider } from "@tonconnect/ui-react";
import React from "react";

export const Button = () => {
    const [counter, setCounter] = React.useState(0);
    return <TonConnectUIProvider>
        <button onClick={() => setCounter(counter + 1)}>Ya react {counter}</button>
    </TonConnectUIProvider>;

};