import { invoke } from '@tauri-apps/api/core'
import type { JsonOptions, JsonOutcome } from './types'

export function jsonProcess(input: string, opts: JsonOptions): Promise<JsonOutcome> {
  return invoke('json_process', { input, opts })
}

export function json2mdConvert(input: string): Promise<string> {
  return invoke('json2md_convert', { input })
}

export function jsonToYaml(input: string): Promise<string> {
  return invoke('json_to_yaml', { input })
}

export function yamlToJson(input: string): Promise<string> {
  return invoke('yaml_to_json', { input })
}

export function hashMd5(input: string): Promise<string> {
  return invoke('hash_md5', { input })
}

export function base64Encode(input: string): Promise<string> {
  return invoke('base64_encode', { input })
}

export function base64Decode(input: string): Promise<string> {
  return invoke('base64_decode', { input })
}

export function unicodeEncode(input: string): Promise<string> {
  return invoke('unicode_encode', { input })
}

export function unicodeDecode(input: string): Promise<string> {
  return invoke('unicode_decode', { input })
}

export interface PasswordOptions {
  length: number
  count: number
  upper: boolean
  lower: boolean
  digits: boolean
  symbols: boolean
  requireEach: boolean
  extraChars: string
  excludeChars: string
}

export function passwordGenerate(opts: PasswordOptions): Promise<string[]> {
  return invoke('password_generate', { opts })
}

export interface UuidOptions {
  count: number
  uppercase: boolean
  hyphens: boolean
  braces: boolean
}

export function uuidGenerate(opts: UuidOptions): Promise<string[]> {
  return invoke('uuid_generate', { opts })
}

export interface JwtDecoded {
  valid: boolean
  error: string | null
  header: string | null
  payload: string | null
  signature: string | null
  issuedAt: number | null
  expiresAt: number | null
  notBefore: number | null
}

export function jwtDecode(token: string): Promise<JwtDecoded> {
  return invoke('jwt_decode', { token })
}

export interface IpInfo {
  ip: string
  ipType: string | null
  continent: string | null
  country: string | null
  countryCode: string | null
  region: string | null
  city: string | null
  postal: string | null
  latitude: number | null
  longitude: number | null
  timezone: string | null
  utcOffset: string | null
  isp: string | null
  org: string | null
  asn: string | null
  source: string
}

export interface IpReport {
  ipv4: IpInfo | null
  ipv6: IpInfo | null
  errors: string[]
}

export function ipLookup(): Promise<IpReport> {
  return invoke('ip_lookup')
}

export function clipboardReadText(): Promise<string> {
  return invoke('clipboard_read_text')
}

export function clipboardWriteText(text: string): Promise<void> {
  return invoke('clipboard_write_text', { text })
}
