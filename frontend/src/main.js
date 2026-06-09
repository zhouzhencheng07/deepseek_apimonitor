import { createApp } from "vue";
import Dashboard from "./Dashboard.vue";
import BallView from "./BallView.vue";
import "./style.css";

const isBall = new URLSearchParams(window.location.search).has("ball");
createApp(isBall ? BallView : Dashboard).mount("#app");
