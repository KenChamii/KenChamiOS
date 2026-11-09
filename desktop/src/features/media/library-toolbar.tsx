import { LayoutGrid, List } from "lucide-react";
import { Button } from "@/components/ui/button";
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
    <div className="flex flex-wrap items-center gap-3">
      <Input
        className="max-w-xs"
        placeholder="Search the library"
        aria-label="Search the library"
        value={filter.search}
        onChange={(e) => onFilter({ ...filter, search: e.target.value })}
      />
      <Select
        aria-label="Filter by type"
        value={filter.type}
        onChange={(e) => onFilter({ ...filter, type: e.target.value as MediaType | "all" })}
      >
        <option value="all">All types</option>
        {TYPES.map((type) => (
          <option key={type.id} value={type.id}>
            {type.label}
          </option>
        ))}
      </Select>
      <Select
        aria-label="Filter by status"
        value={filter.status}
        onChange={(e) => onFilter({ ...filter, status: e.target.value as MediaStatus | "all" })}
      >
        <option value="all">All statuses</option>
        {STATUSES.map((status) => (
          <option key={status} value={status}>
            {statusLabels[status]}
          </option>
        ))}
      </Select>
      <Select
        aria-label="Filter by genre"
        value={filter.genre}
        onChange={(e) => onFilter({ ...filter, genre: e.target.value })}
      >
        <option value="">All genres</option>
        {genres.map((genre) => (
          <option key={genre} value={genre}>
            {genre}
          </option>
        ))}
      </Select>
      <Select
        aria-label="Filter by tag"
        value={filter.tag}
        onChange={(e) => onFilter({ ...filter, tag: e.target.value })}
      >
        <option value="">All tags</option>
        {tags.map((tag) => (
          <option key={tag} value={tag}>
            {tag}
          </option>
        ))}
      </Select>
      <Select
        aria-label="Sort by"
        value={filter.sort}
        onChange={(e) => onFilter({ ...filter, sort: e.target.value as SortKey })}
      >
        {(Object.keys(SORT_LABELS) as SortKey[]).map((key) => (
          <option key={key} value={key}>
            {SORT_LABELS[key]}
          </option>
        ))}
      </Select>
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
  );
}
