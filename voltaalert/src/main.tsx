import ReactDOM from "react-dom/client";
import App from "./App";

// Avoid React.StrictMode: listen may be called twice
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <App />
);
