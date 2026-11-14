import { mount } from "svelte";
import { waitLocale } from "svelte-i18n";
import { setupI18n } from "../i18n/index.ts";
import SubtitleApp from "./SubtitleApp.svelte";

setupI18n();
await waitLocale();
mount(SubtitleApp, { target: document.getElementById("app")! });
