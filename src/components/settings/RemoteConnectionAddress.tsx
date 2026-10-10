import { useMemo } from "react";
import QRCode from "qrcode";
import type { RemoteAddress } from "../../types/remote";

interface Props {
  address: RemoteAddress;
  token: string;
  onCopy: (link: string) => void;
}

export function RemoteConnectionAddress({ address, token, onCopy }: Props) {
  const link = `${address.url}/#key=${token}`;
  const qr = useMemo(() => {
    try {
      const { modules } = QRCode.create(link, { errorCorrectionLevel: "M" });
      const size = modules.size;
      let path = "";
      for (let row = 0; row < size; row++) {
        for (let column = 0; column < size; column++) {
          if (modules.get(row, column))
            path += `M${column + 4} ${row + 4}h1v1h-1z`;
        }
      }
      return { size: size + 8, path };
    } catch {
      return null;
    }
  }, [link]);
  const label = address.url.includes("127.0.0.1")
    ? "This computer only"
    : address.local_network
      ? "Wi-Fi / Ethernet"
      : "Other network";

  return (
    <div className="remote-address">
      <div className="remote-address-content">
        <p className="form-label">
          {label}
          {address.interface_name && ` · ${address.interface_name}`}
        </p>
        <input
          aria-label={`${label} connection address`}
          className="form-input"
          readOnly
          value={address.url}
        />
        <button className="btn-primary" onClick={() => onCopy(link)}>
          Copy private link
        </button>
        <p className="form-help">
          {address.url.includes("127.0.0.1")
            ? "For testing on this computer; phones cannot use this address."
            : "Scan with your phone’s camera to connect on this network."}
        </p>
      </div>
      {qr ? (
        <svg
          className="remote-connection-qr"
          viewBox={`0 0 ${qr.size} ${qr.size}`}
          role="img"
          aria-label={`Private connection QR code for ${address.url}`}
          shapeRendering="crispEdges"
        >
          <rect width={qr.size} height={qr.size} fill="#fff" />
          <path d={qr.path} fill="#000" />
        </svg>
      ) : (
        <p className="form-help">QR code unavailable. Use the private link.</p>
      )}
    </div>
  );
}
