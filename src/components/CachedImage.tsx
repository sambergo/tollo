import React, { useRef, useEffect, useState } from "react";
import { useCachedImageAsync } from "../hooks/useImageCache";

interface CachedImageProps {
  src: string;
  alt: string;
  className?: string;
  style?: React.CSSProperties;
  onLoad?: () => void;
  onError?: () => void;
  lazy?: boolean; // Enable lazy loading
  rootMargin?: string; // Intersection observer root margin
}

const CachedImage: React.FC<CachedImageProps> = ({
  src,
  alt,
  className,
  style,
  onLoad,
  onError,
  lazy = true,
  rootMargin = "50px",
}) => {
  const [isIntersecting, setIsIntersecting] = useState(!lazy);
  const imgRef = useRef<HTMLDivElement | HTMLImageElement>(null);

  useEffect(() => {
    if (!lazy || !imgRef.current) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsIntersecting(true);
          observer.disconnect();
        }
      },
      { rootMargin },
    );

    observer.observe(imgRef.current);

    return () => observer.disconnect();
  }, [lazy, rootMargin]);

  // Use the new async hook for non-blocking image caching
  const { cachedUrl, loading, error } = useCachedImageAsync(
    src,
    isIntersecting,
  );

  // Show placeholder for empty URLs
  if (!src || src.trim() === "") {
    return (
      <div
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className}
        style={{
          ...style,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#27272a",
          color: "#71717a",
          minWidth: "40px",
          minHeight: "40px",
          fontSize: "10px",
          borderRadius: "8px",
        }}
      >
        No Logo
      </div>
    );
  }

  // Show placeholder while not intersecting (lazy loading)
  if (lazy && !isIntersecting) {
    return (
      <div
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className}
        style={{
          ...style,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#27272a",
          color: "#71717a",
          minWidth: "40px",
          minHeight: "40px",
          fontSize: "16px",
          borderRadius: "8px",
        }}
      >
        📷
      </div>
    );
  }

  // Show loading state while image is being cached asynchronously
  if (loading) {
    return (
      <div
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className}
        style={{
          ...style,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#27272a",
          color: "#a1a1aa",
          minWidth: "40px",
          minHeight: "40px",
          fontSize: "10px",
          borderRadius: "8px",
        }}
      >
        Loading...
      </div>
    );
  }

  // Show error state if there's an error
  if (error) {
    return (
      <div
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className}
        style={{
          ...style,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#27272a",
          color: "#ef4444",
          minWidth: "40px",
          minHeight: "40px",
          fontSize: "10px",
          borderRadius: "8px",
        }}
      >
        Error
      </div>
    );
  }

  // Show placeholder for empty cached URLs
  if (!cachedUrl || cachedUrl.trim() === "") {
    return (
      <div
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className}
        style={{
          ...style,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#27272a",
          color: "#71717a",
          minWidth: "40px",
          minHeight: "40px",
          fontSize: "10px",
          borderRadius: "8px",
        }}
      >
        No Logo
      </div>
    );
  }

  return (
    <img
      ref={imgRef as React.RefObject<HTMLImageElement>}
      src={cachedUrl}
      alt={alt}
      className={className}
      style={style}
      onLoad={onLoad}
      onError={onError}
    />
  );
};

export default CachedImage;
