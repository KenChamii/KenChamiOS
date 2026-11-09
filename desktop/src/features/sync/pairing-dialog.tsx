import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useCancelPairing, useStartPairing } from "@/hooks/use-sync";
import { errorMessage } from "@/lib/errors";
import type { PairingInfo } from "@/lib/api/sync";
import { countdown, groupFingerprint, pairingText } from "./pairing-text";

export function PairingDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const start = useStartPairing();
  const cancel = useCancelPairing();
  const { mutate: startPairing } = start;
  const { mutate: cancelPairing } = cancel;

  useEffect(() => {
    if (!open) return;
    startPairing();
    return () => cancelPairing();
  }, [open, startPairing, cancelPairing]);

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent className="max-w-lg">
        <DialogTitle className="text-lg font-semibold">Pair a phone</DialogTitle>
        <DialogDescription className="mt-1 text-sm text-muted-foreground">
          Open KenChamiOS on the phone, choose Pair and scan this code. It works once and expires in
          a few minutes.
        </DialogDescription>
        {start.error && (
          <p className="mt-3 text-sm text-destructive">{errorMessage(start.error)}</p>
        )}
        {start.data && <PairingDetails info={start.data} onExpired={onClose} />}
        <div className="mt-4 flex justify-end">
          <Button variant="outline" onClick={onClose}>
            Close
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function PairingDetails({ info, onExpired }: { info: PairingInfo; onExpired: () => void }) {
  const [left, setLeft] = useState(info.expires_in_s);

  useEffect(() => {
    const timer = window.setInterval(() => setLeft((value) => value - 1), 1000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (left <= 0) onExpired();
  }, [left, onExpired]);

  const { payload } = info;
  return (
    <div className="mt-4 flex flex-col items-center gap-3">
      <img
        alt="Pairing QR code"
        className="h-56 w-56 rounded bg-white p-2"
        src={`data:image/svg+xml;utf8,${encodeURIComponent(info.qr_svg)}`}
      />
      <p className="text-sm">
        Expires in <span className="font-medium">{countdown(left)}</span>
      </p>
      <dl className="grid w-full grid-cols-[6rem_1fr] gap-2 text-sm">
        <dt className="text-muted-foreground">Address</dt>
        <dd>
          {payload.host}:{payload.port}
        </dd>
        <dt className="text-muted-foreground">Code</dt>
        <dd className="font-mono text-base tracking-widest">{payload.code}</dd>
        <dt className="text-muted-foreground">Fingerprint</dt>
        <dd className="break-all font-mono text-xs">{groupFingerprint(payload.fp)}</dd>
      </dl>
      <details className="w-full text-sm">
        <summary className="cursor-pointer text-muted-foreground">
          No camera? Type this text on the phone
        </summary>
        <textarea
          readOnly
          className="mt-2 h-20 w-full rounded border border-border bg-muted p-2 font-mono text-xs"
          value={pairingText(payload)}
          onFocus={(event) => event.currentTarget.select()}
        />
      </details>
    </div>
  );
}
