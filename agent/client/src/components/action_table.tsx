"use client"

import type React from "react"

import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { Trash2 } from "lucide-react"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import type { ActionDto } from "../lib/types"
import { deleteAction } from "../lib/api"
interface ActionTableProps {
  actions: ActionDto[]
}

export function ActionTable({ actions }: ActionTableProps) {
  const navigate = useNavigate()
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [actionToDelete, setActionToDelete] = useState<number | null>(null)

  const handleRowClick = (id: number) => {
    navigate(`/actions/${id}`)
  }

  const handleDeleteClick = (e: React.MouseEvent<HTMLButtonElement>, id: number) => {
    e.stopPropagation()
    setActionToDelete(id)
    setIsDeleteDialogOpen(true)
  }

  const confirmDelete = async () => {
    if (actionToDelete) {
      try {
        await deleteAction(actionToDelete)
        // We don't need to update the state here as the WebSocket will handle it
      } catch (error) {
        console.error("Failed to delete action:", error)
      }
    }
    setIsDeleteDialogOpen(false)
    setActionToDelete(null)
  }

  const getStateBadge = (state: string) => {
    switch (state.toLowerCase()) {
      case "running":
      case "in progress":
      case "inprogress":
        return (
          <Badge variant="outline" className="bg-orange-100 text-orange-800 border-orange-300">
            In Progress
          </Badge>
        )
      case "completed":
      case "success":
        return (
          <Badge variant="outline" className="bg-green-100 text-green-800 border-green-300">
            Completed
          </Badge>
        )
      case "failed":
      case "error":
        return (
          <Badge variant="outline" className="bg-red-100 text-red-800 border-red-300">
            Failed
          </Badge>
        )
      default:
        return (
          <Badge variant="outline" className="bg-gray-100 text-gray-800 border-gray-300">
            {state}
          </Badge>
        )
    }
  }

  return (
    <>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[100px]">ID</TableHead>
              <TableHead>Repository</TableHead>
              <TableHead>Image</TableHead>
              <TableHead>Status</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {actions.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="h-24 text-center">
                  <div className="flex flex-col items-center justify-center py-4">
                    <p className="text-muted-foreground mb-2">No actions found</p>
                    <p className="text-sm text-muted-foreground">Create a new action to get started</p>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              actions.map((action) => (
                <TableRow
                  key={action.id}
                  className="cursor-pointer hover:bg-muted/50"
                  onClick={() => handleRowClick(action.id)}
                >
                  <TableCell className="font-medium">{action.id}</TableCell>
                  <TableCell className="max-w-[300px] truncate">{action.repo_url}</TableCell>
                  <TableCell>{action.image}</TableCell>
                  <TableCell>{getStateBadge(action.state)}</TableCell>
                  <TableCell className="text-right">
                    <Button variant="ghost" size="icon" onClick={(e) => handleDeleteClick(e, action.id)}>
                      <Trash2 className="h-4 w-4" />
                      <span className="sr-only">Delete</span>
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <AlertDialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
            <AlertDialogDescription>
              This action will permanently delete this CI/CD action and cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDelete}>Delete</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

