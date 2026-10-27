import { createApp } from "vue";
import "./styles/theme.css";
import "./styles/app.css";
import App from "./App.vue";
import { initTheme } from "./theme";

initTheme();

createApp(App).mount("#app");
