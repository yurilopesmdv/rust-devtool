import { Registry, Counter, Histogram, collectDefaultMetrics } from 'prom-client';
export const registry = new Registry();
collectDefaultMetrics({ register: registry });
const requests = new Counter({ name: 'http_requests_total', help: 'Requisições HTTP', labelNames: ['method','route','status'], registers: [registry] });
const duration = new Histogram({ name: 'http_request_duration_seconds', help: 'Duração HTTP', labelNames: ['method','route'], buckets: [0.005,0.01,0.025,0.05,0.1,0.25,0.5,1,2.5,5], registers: [registry] });
// Interface estrutural compartilhada pelos adapters Express e Nest/Express.
export function metricsMiddleware(req: {method:string,route?:{path?:string}}, res: {statusCode:number,on:(event:string,listener:()=>void)=>unknown}, next:()=>void) {
  const start = process.hrtime.bigint();
  res.on('finish', () => {
    const route = typeof req.route?.path === 'string' ? req.route.path : 'unmatched';
    if (route === '/metrics') return;
    const method = ['GET','POST','PUT','PATCH','DELETE','OPTIONS','HEAD'].includes(req.method) ? req.method : 'OTHER';
    requests.inc({ method, route, status: String(res.statusCode) });
    duration.observe({ method, route }, Number(process.hrtime.bigint()-start)/1e9);
  });
  next();
}
export async function metricsHandler(_req: unknown, res: {setHeader:(name:string,value:string)=>unknown,end:(body:string)=>unknown}, next?:(error:unknown)=>void) {
  try { res.setHeader('Content-Type', registry.contentType); res.end(await registry.metrics()); } catch(error) { if(next) next(error); else res.end('metrics unavailable'); }
}
