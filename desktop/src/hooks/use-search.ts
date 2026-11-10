import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { searchAll } from "@/lib/api/search";

const MIN_CHARS = 2;

/** Full-text hits for the palette; nothing until two characters are typed. */
export function useSearch(query: string) {
  const text = query.trim();
  return useQuery({
    queryKey: ["search", text],
    queryFn: () => searchAll(text),
    enabled: text.length >= MIN_CHARS,
    placeholderData: keepPreviousData,
  });
}
