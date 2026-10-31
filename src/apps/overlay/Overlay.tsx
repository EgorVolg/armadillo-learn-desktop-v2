import { invoke } from '@tauri-apps/api/core';

export const Overlay = () => {
  const handleClick = () => {
    invoke('log_hello');
  };

  return <div onClick={handleClick}>Overlay</div>;
};
