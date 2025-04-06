export interface ActionDto {
  id: number
  state: string
  repo_url: string
  image: string
}


export interface CreateActionRequest {
  image: string
  commands: string[]
  repo_url: string
  action_id: number
}

export interface DeleteActionResponse {
  id: number
}

export interface StateEvent {
  action_id: number
  state: string
  timestamp: string
}

export interface ErrorResponse {
  error: string
  code?: number
}

