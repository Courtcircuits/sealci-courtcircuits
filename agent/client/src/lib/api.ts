import { ActionDto, CreateActionRequest, DeleteActionResponse } from "./types"

const API_BASE_URL = "http://localhost:8080"

export async function fetchActions(): Promise<ActionDto[]> {
  try {
    console.log("test")

    const response = await fetch(`${API_BASE_URL}/actions`)
    console.log(response.body)

    if (!response.ok) {
      const error = await response.json()
      throw new Error(error.error || "Failed to fetch actions")
    }
    return response.json()
  } catch (error) {
    console.error("API connection error:", error)
    // Return empty array instead of throwing
    return []
  }
}

export async function fetchActionById(id: number): Promise<ActionDto | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/actions/${id}`)

    if (!response.ok) {
      const error = await response.json()
      throw new Error(error.error || `Failed to fetch action with ID ${id}`)
    }

    return response.json()
  } catch (error) {
    console.error(`API connection error for action ${id}:`, error)
    return null
  }
}

export async function createAction(data: CreateActionRequest): Promise<ActionDto> {
  const response = await fetch(`${API_BASE_URL}/actions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })

  if (!response.ok) {
    const error = await response.json()
    throw new Error(error.error || "Failed to create action")
  }

  return response.json()
}

export async function deleteAction(id: number): Promise<DeleteActionResponse> {
  const response = await fetch(`${API_BASE_URL}/actions/${id}`, {
    method: "DELETE",
  })

  if (!response.ok) {
    const error = await response.json()
    throw new Error(error.error || `Failed to delete action with ID ${id}`)
  }

  return response.json()
}

