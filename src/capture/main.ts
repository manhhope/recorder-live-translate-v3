import { mount } from "svelte";
import { waitLocale } from "svelte-i18n";
import { setupI18n } from "../i18n/index.ts";
import CaptureApp from "./CaptureApp.svelte";

setupI18n();
await waitLocale();
mount(CaptureApp, { target: document.getElementById("app")! });
