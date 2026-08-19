/** Generated-shape declaration maintained alongside the MP10 WASM build seam. */
export class EngineHandle {
  analyzeFrame(image: Uint8Array, width: number, height: number, stride: number, pixelFormat: string, requestJson: string): string;
  reconstructPage(image: Uint8Array, width: number, height: number, stride: number, pixelFormat: string, requestJson: string): string;
  applyFilter(image: Uint8Array, width: number, height: number, stride: number, pixelFormat: string, requestJson: string): string;
}
export function createEngine(configJson?: string): EngineHandle;
export function destroyEngine(handle: EngineHandle): void;
export function version(): string;
