import React, { useRef, useEffect, useState } from 'react';
import { useCachedImage } from '../hooks/useImageCache';

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
  rootMargin = '50px'
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
      { rootMargin }
    );
    
    observer.observe(imgRef.current);
    
    return () => observer.disconnect();
  }, [lazy, rootMargin]);

  const { cachedUrl, loading, error } = useCachedImage(src, isIntersecting);

  // Show placeholder for empty URLs
  if (!src || src.trim() === '') {
    return (
      <div 
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className} 
        style={{ 
          ...style, 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          backgroundColor: '#e0e0e0',
          color: '#757575',
          minWidth: '50px',
          minHeight: '50px',
          fontSize: '12px'
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
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          backgroundColor: '#f5f5f5',
          color: '#9e9e9e',
          minWidth: '50px',
          minHeight: '50px',
          fontSize: '12px'
        }}
      >
        📷
      </div>
    );
  }

  // Show a placeholder or loading state if needed
  if (loading) {
    return (
      <div 
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className} 
        style={{ 
          ...style, 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          backgroundColor: '#f0f0f0',
          color: '#666',
          minWidth: '50px',
          minHeight: '50px',
          fontSize: '12px'
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
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          backgroundColor: '#ffebee',
          color: '#c62828',
          minWidth: '50px',
          minHeight: '50px',
          fontSize: '12px'
        }}
      >
        Error
      </div>
    );
  }

  // Show placeholder for empty cached URLs
  if (!cachedUrl || cachedUrl.trim() === '') {
    return (
      <div 
        ref={imgRef as React.RefObject<HTMLDivElement>}
        className={className} 
        style={{ 
          ...style, 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          backgroundColor: '#e0e0e0',
          color: '#757575',
          minWidth: '50px',
          minHeight: '50px',
          fontSize: '12px'
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