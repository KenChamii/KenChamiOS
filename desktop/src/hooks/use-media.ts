import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  addManualMedia,
  addMedia,
  clearGamesKey,
  clearTmdbToken,
  deleteMedia,
  gamesKeyStatus,
  listMedia,
  searchMedia,
  setGamesKey,
  setTmdbToken,
  setMediaProgress,
  tmdbTokenStatus,
  updateMedia,
  type EntryInput,
  type MediaStatus,
  type MediaType,
} from "@/lib/api/media";

export function useMedia() {
  return useQuery({ queryKey: ["media"], queryFn: listMedia });
}

function useInvalidateMedia() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["media"] });
}

// the query is searched on request, so it only runs while a non-empty text is set
export function useMediaSearch(query: string, mediaType: MediaType) {
  return useQuery({
    queryKey: ["media-search", mediaType, query],
    queryFn: () => searchMedia(query, mediaType),
    enabled: query.trim().length > 0,
    retry: false,
    staleTime: Infinity,
  });
}

export function useAddMedia() {
  const invalidate = useInvalidateMedia();
  return useMutation({
    mutationFn: (args: { mediaType: MediaType; externalId: string; status: MediaStatus }) =>
      addMedia(args.mediaType, args.externalId, args.status),
    onSuccess: invalidate,
  });
}

export function useAddManualMedia() {
  const invalidate = useInvalidateMedia();
  return useMutation({ mutationFn: addManualMedia, onSuccess: invalidate });
}

export function useUpdateMedia() {
  const invalidate = useInvalidateMedia();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: EntryInput }) => updateMedia(id, input),
    onSuccess: invalidate,
  });
}

export function useSetMediaProgress() {
  const invalidate = useInvalidateMedia();
  return useMutation({
    mutationFn: ({ id, value }: { id: string; value: number }) => setMediaProgress(id, value),
    onSuccess: invalidate,
  });
}

export function useDeleteMedia() {
  const invalidate = useInvalidateMedia();
  return useMutation({ mutationFn: deleteMedia, onSuccess: invalidate });
}

export function useGamesKeyStatus() {
  return useQuery({ queryKey: ["games-key"], queryFn: gamesKeyStatus });
}

export function useSetGamesKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: setGamesKey,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["games-key"] }),
  });
}

export function useClearGamesKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: clearGamesKey,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["games-key"] }),
  });
}

export function useTmdbTokenStatus() {
  return useQuery({ queryKey: ["tmdb-token"], queryFn: tmdbTokenStatus });
}

export function useSetTmdbToken() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: setTmdbToken,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tmdb-token"] }),
  });
}

export function useClearTmdbToken() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: clearTmdbToken,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tmdb-token"] }),
  });
}
