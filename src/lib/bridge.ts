import { invoke } from "@tauri-apps/api/core";

export type RuntimeStatus = {
  platform: string;
  arch: string;
  session: string;
  dataDir: string;
  umuAvailable: boolean;
  umuManaged: boolean;
  umuVersion: string | null;
  pythonVersion: string | null;
  pythonCompatible: boolean;
  hoyoplayInstalled: boolean;
  vulkanAvailable: boolean;
};

export type BootstrapResult = {
  version: string;
  path: string;
  updated: boolean;
};

export const defaultStatus: RuntimeStatus = {
  platform: "Linux",
  arch: "Detecting",
  session: "Detecting",
  dataDir: "~/.local/share/lumina",
  umuAvailable: false,
  umuManaged: false,
  umuVersion: null,
  pythonVersion: null,
  pythonCompatible: false,
  hoyoplayInstalled: false,
  vulkanAvailable: false,
};

export async function getSystemStatus(): Promise<RuntimeStatus> {
  return invoke<RuntimeStatus>("get_system_status");
}

export async function prepareLumina(): Promise<string> {
  return invoke<string>("ensure_layout");
}

export async function bootstrapUmu(): Promise<BootstrapResult> {
  return invoke<BootstrapResult>("bootstrap_umu");
}

export async function launchHoyoplay(): Promise<string> {
  return invoke<string>("launch_hoyoplay");
}
