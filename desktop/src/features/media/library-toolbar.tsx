import { LayoutGrid, List } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ChipGroup } from "@/components/ui/chips";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import type { MediaStatus, MediaType } from "@/lib/api/media";
import type { LibraryFilter, SortKey } from "./library-filter";
import { GENERIC_STATUS_LABELS, SORT_LABELS, STATUS_LABELS, STATUSES, TYPES } from "./meta";

export type ViewMode = "grid" | "list";

type LibraryToolbarProps = {
  filter: LibraryFilter;
  onFilter: (filter: LibraryFilter) => void;
  genres: string[];
  tags: string[];
  view: ViewMode;
  onView: (view: ViewMode) => void;
};

export function LibraryToolbar({
  filter,
  onFilter,
  genres,
  tags,
  view,
  onView,
}: LibraryToolbarProps) {
  const statusLabels = filter.type === "all" ? GENERIC_STATUS_LABELS : STATUS_LABELS[filter.type];

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <Input
          className="max-w-xs"
          placeholder="Search the library"
          aria-label="Search the library"
          value={filter.search}
          onChange={(e) => onFilter({ ...filter, search: e.target.value })}
        />
        <Select
          aria-label="Filter by genre"
          className="w-40"
          value={filter.genre}
          options={[
            { value: "", label: "All genres" },
            ...genres.map((genre) => ({ value: genre, label: genre })),
          ]}
          onChange={(genre) => onFilter({ ...filter, genre })}
        />
        <Select
          aria-label="Filter by tag"
          className="w-40"
          value={filter.tag}
          options={[
            { value: "", label: "All tags" },
            ...tags.map((tag) => ({ value: tag, label: tag })),
          ]}
          onChange={(tag) => onFilter({ ...filter, tag })}
        />
        <Select
          aria-label="Sort by"
          className="w-44"
          value={filter.sort}
          options={(Object.keys(SORT_LABELS) as SortKey[]).map((key) => ({
            value: key,
            label: SORT_LABELS[key],
          }))}
          onChange={(sort) => onFilter({ ...filter, sort })}
        />
        <div className="ml-auto flex gap-1">
          <Button
            size="icon"
            variant={view === "grid" ? "outline" : "ghost"}
            aria-label="Cover grid"
            aria-pressed={view === "grid"}
            onClick={() => onView("grid")}
          >
            <LayoutGrid size={16} />
          </Button>
          <Button
            size="icon"
            variant={view === "list" ? "outline" : "ghost"}
            aria-label="List"
            aria-pressed={view === "list"}
            onClick={() => onView("list")}
          >
            <List size={16} />
          </Button>
        </div>
      </div>
      <ChipGroup<MediaType | "all">
        aria-label="Filter by type"
        value={filter.type}
        options={[
          { value: "all", label: "All types" },
          ...TYPES.map((type) => ({ value: type.id, label: type.label })),
        ]}
        onChange={(type) => onFilter({ ...filter, type })}
      />
      <ChipGroup<MediaStatus | "all">
        aria-label="Filter by status"
        value={filter.status}
        options={[
          { value: "all", label: "Any status" },
          ...STATUSES.map((status) => ({ value: status, label: statusLabels[status] })),
        ]}
        onChange={(status) => onFilter({ ...filter, status })}
      />
    </div>
  );
}
