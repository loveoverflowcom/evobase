namespace EvobaseDemo

inductive OrderState where
  | draft
  | paid
  | shipped
  | cancelled
  deriving DecidableEq, Repr

inductive Transition : OrderState -> OrderState -> Prop where
  | pay : Transition .draft .paid
  | ship : Transition .paid .shipped
  | cancelDraft : Transition .draft .cancelled
  | cancelPaid : Transition .paid .cancelled

inductive Reaches : OrderState -> OrderState -> Prop where
  | refl (state : OrderState) : Reaches state state
  | step {source middle target : OrderState} :
      Transition source middle -> Reaches middle target -> Reaches source target

theorem no_direct_ship_from_draft : Not (Transition .draft .shipped) := by
  intro transition
  cases transition

theorem shipped_direct_source_is_paid {source : OrderState}
    (transition : Transition source .shipped) : source = .paid := by
  cases transition
  rfl

theorem shipped_is_terminal : Not (Exists fun target => Transition .shipped target) := by
  intro existsTransition
  cases existsTransition with
  | intro target transition =>
      cases transition

theorem cancelled_is_terminal : Not (Exists fun target => Transition .cancelled target) := by
  intro existsTransition
  cases existsTransition with
  | intro target transition =>
      cases transition

theorem draft_reaches_shipped : Reaches .draft .shipped := by
  exact Reaches.step Transition.pay
    (Reaches.step Transition.ship (Reaches.refl .shipped))

end EvobaseDemo
