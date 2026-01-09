declare module "../../pkg/palmietopia_core" {
  export default function init(path?: string): Promise<void>;
  export function get_welcome_message(): string;
  export function generate_tiny_map(): string;
}
