import { convertFileSrc } from "@tauri-apps/api/core";
import { ImageOff } from "lucide-react";
import { cn } from "@/lib/cn";

type CoverProps = {
  title: string;
  /** cached file on this device, preferred so covers show offline */
  path: string | null;
  url: string | null;
  className?: string;
};

export function Cover({ title, path, url, className }: CoverProps) {
  const src = path ? convertFileSrc(path) : url;
  return (
    <div
      className={cn(
        "flex aspect-[2/3] items-center justify-center overflow-hidden rounded-xl bg-muted",
        className,
      )}
    >
      {src ? (
        <img src={src} alt={title} loading="lazy" className="h-full w-full object-cover" />
      ) : (
        <ImageOff size={20} className="text-muted-foreground" aria-label="No cover" />
      )}
    </div>
  );
}
