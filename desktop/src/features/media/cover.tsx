import { convertFileSrc } from "@tauri-apps/api/core";
import type { MediaType } from "@/lib/api/media";
import { cn } from "@/lib/cn";
import { TYPE_ICONS } from "./meta";

type CoverProps = {
  title: string;
  /** cached file on this device, preferred so covers show offline */
  path: string | null;
  url: string | null;
  /** its icon stands in when there is no image */
  mediaType: MediaType;
  className?: string;
};

export function Cover({ title, path, url, mediaType, className }: CoverProps) {
  const src = path ? convertFileSrc(path) : url;
  const Fallback = TYPE_ICONS[mediaType];
  return (
    <div
      className={cn(
        "flex aspect-[2/3] items-center justify-center overflow-hidden rounded-xl",
        src ? "bg-muted" : "bg-media-soft text-media",
        className,
      )}
    >
      {src ? (
        <img src={src} alt={title} loading="lazy" className="h-full w-full object-cover" />
      ) : (
        <Fallback size={22} aria-label="No cover" />
      )}
    </div>
  );
}
