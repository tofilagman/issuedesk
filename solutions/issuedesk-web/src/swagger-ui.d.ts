// swagger-ui ships no bundled types; we use only its default factory.
declare module 'swagger-ui' {
  const SwaggerUI: (opts: Record<string, unknown>) => unknown;
  export default SwaggerUI;
}
declare module 'swagger-ui/dist/swagger-ui.css';
