import { mount } from "svelte";
import { waitLocale } from "svelte-i18n";
import "../styles/app.css";
import AskAiWindow from "./AskAiWindow.svelte";
import { setupI18n } from "../i18n/index.ts";

setupI18n();
await waitLocale();
export default mount(AskAiWindow, { target: document.getElementById("app")! });
