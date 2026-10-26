package example

import "context"

type Example struct {
	Message string `json:"message"`
}
type Service struct{}

func NewService() *Service                                { return &Service{} }
func (s *Service) Get(_ context.Context) (Example, error) { return Example{Message: "Olá do Go"}, nil }
