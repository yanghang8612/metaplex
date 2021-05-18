import React from 'react';
import { useParams } from 'react-router-dom';
import { useCreator } from '../../hooks';

export const ArtistView = () => {
  const { id } = useParams<{ id: string }>();
  const creator = useCreator(id);
  return (
    <div className="flexColumn" style={{ flex: 1 }}>
      {creator?.info.name || creator?.info.address.toBase58()}
    </div>
  );
};
