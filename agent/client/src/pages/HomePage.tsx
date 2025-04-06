
import { useEffect, useState } from "react"
import { useNavigate } from "react-router-dom"
import { Plus } from "lucide-react"
import { Button } from "@/components/ui/button"
import { ActionTable } from "../components/action_table"
import type { ActionDto } from "../lib/types"
import { fetchActions } from "../lib/api"

export default function HomePage() {
  const [actions, setActions] = useState<ActionDto[]>([])
  const [loading, setLoading] = useState(true)
  const navigate = useNavigate()

  useEffect(() => {
    const loadActions = async () => {
      try {
        const data = await fetchActions()
        setActions(data)
      } catch (error) {
        console.error("Failed to fetch actions:", error)
      } finally {
        setLoading(false)
      }
    }

    loadActions()
  }, [])

  // // WebSocket for new actions
  // useWebSocket("ws://localhost:8080/actions/stream", (event) => {
  //   try {
  //     const newAction = JSON.parse(event.data) as ActionDto
  //     setActions((prevActions) => [...prevActions, newAction])
  //   } catch (error) {
  //     console.error("Error processing WebSocket message:", error)
  //   }
  // })

  // // WebSocket for action state changes
  // useWebSocket("ws://localhost:8080/actions/state/stream", (event) => {
  //   try {
  //     const stateEvent = JSON.parse(event.data)
  //     setActions((prevActions) =>
  //       prevActions.map((action) =>
  //         action.id === stateEvent.action_id ? { ...action, state: stateEvent.state } : action,
  //       ),
  //     )
  //   } catch (error) {
  //     console.error("Error processing WebSocket message:", error)
  //   }
  // })

  return (
    <div className="container mx-auto py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">SealCI Actions</h1>
        <Button onClick={() => navigate("/actions/new")}>
          <Plus className="mr-2 h-4 w-4" /> New Action
        </Button>
      </div>

      {loading ? (
        <div className="flex justify-center items-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
        </div>
      ) : (
        <ActionTable actions={actions} />
      )}
    </div>
  )
}

