import { useQueryClient } from "@tanstack/react-query";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { errorMessage } from "@/lib/errors";
import { useToasts, type Toast } from "@/stores/toasts";

const VISIBLE_MS = 6000;

export function Toaster() {
  const toast = useToasts((state) => state.toast);
  if (!toast) return null;
  // keyed, so a new toast starts its own timer
  return <ToastView key={toast.id} toast={toast} />;
}

function ToastView({ toast }: { toast: Toast }) {
  const queryClient = useQueryClient();
  const { show, dismiss } = useToasts();
  const [pending, setPending] = useState(false);

  useEffect(() => {
    const timer = window.setTimeout(() => dismiss(toast.id), VISIBLE_MS);
    return () => window.clearTimeout(timer);
  }, [toast.id, dismiss]);

  async function undo() {
    if (!toast.undo || pending) return;
    setPending(true);
    try {
      await toast.undo();
      dismiss(toast.id);
    } catch (error) {
      show(`Could not undo: ${errorMessage(error)}`);
    } finally {
      // an undo can touch any list, so everything refreshes
      void queryClient.invalidateQueries();
    }
  }

  return (
    <div
      role="status"
      className="fixed bottom-6 left-1/2 z-50 flex -translate-x-1/2 animate-pop-in items-center gap-3 rounded-2xl bg-card py-2 pl-4 pr-2 text-sm shadow-soft"
    >
      <span>{toast.message}</span>
      {toast.undo && (
        <Button size="sm" variant="outline" disabled={pending} onClick={() => void undo()}>
          Undo
        </Button>
      )}
      <Button size="icon" variant="ghost" aria-label="Dismiss" onClick={() => dismiss(toast.id)}>
        <X size={14} />
      </Button>
    </div>
  );
}
