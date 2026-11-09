import { invoke } from "@tauri-apps/api/core";

export type AppInfo = {
  version: string;
  device_id: string;
  db_path: string;
};

export type AppError = {
  code: string;
  message: string;
};

export function getAppInfo() {
  return invoke<AppInfo>("app_info");
}
