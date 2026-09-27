import { invoke } from "@tauri-apps/api/core";

export type RuntimeStatus = {
  platform: string;
  arch: string;
  session: string;
  dataDir: string;
  umuAvailable: boolean;
  hoyoplayInstalled: boolean;
  vulkanAvailable: boolean;
};

export const defaultStatus: RuntimeStatus = {
  platform: "Linux",
  arch: "Detecting",
  session: "Detecting",
  dataDir: "~/.local/share/lumina",
  umuAvailable: false,
  hoyoplayInstalled: false,
  vulkanAvailable: false,
};

export async function getSystemStatus(): Promise<RuntimeStatus> {
  return invoke<RuntimeStatus>("get_system_status");
}

export async function prepareLumina(): Promise<string> {
  return invoke<string>("ensure_layout");
}

export async function launchHoyoplay(): Promise<string> {
  return invoke<string>("launch_hoyoplay");
}
