import { mount } from "svelte";
import { waitLocale } from "svelte-i18n";
import "../styles/app.css";
import QuickWindow from "./QuickWindow.svelte";
import Toolbar from "./Toolbar.svelte";
import { setupI18n } from "../i18n/index.ts";

setupI18n();
await waitLocale();
const view = new URLSearchParams(location.search).get("view");
export default mount(view === "toolbar" ? Toolbar : QuickWindow, { target: document.getElementById("app")! });
