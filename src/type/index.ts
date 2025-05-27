export type ErrorKind = {
  kind: 'io' | 'utf8' | 'invalidIpAddr' | 'server';
  message: string;
};

