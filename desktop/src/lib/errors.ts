import type { AppError } from "@/lib/api/app";

function isAppError(error: unknown): error is AppError {
  return typeof error === "object" && error !== null && "message" in error;
}

export function errorMessage(error: unknown): string {
  if (isAppError(error)) return String(error.message);
  return typeof error === "string" ? error : "Something went wrong";
}
