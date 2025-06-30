import React from 'react';
import { useCachedImage } from '../hooks/useImageCache';

interface CachedImageProps {
  src: string;
  alt: string;
  className?: string;
  style?: React.CSSProperties;
  onLoad?: () => void;
  onError?: () => void;
}

const CachedImage: React.FC<CachedImageProps> = ({ 
  src, 
  alt, 
  className, 
  style, 
  onLoad, 
  onError 
}) => {
  const { cachedUrl, loading, error } = useCachedImage(src);

  // Show placeholder for empty URLs
  if (!src || src.trim() === '') {
    return (
      <div 
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

  // Show a placeholder or loading state if needed
  if (loading) {
    return (
      <div 
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