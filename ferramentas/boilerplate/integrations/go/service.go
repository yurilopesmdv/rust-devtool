package example

import (
	"context"
	"errors"
)

type Example struct {
	Message string `json:"message"`
}
type Repository interface {
	Read(context.Context) (string, error)
	Write(context.Context, string) error
}
type Service struct{ repository Repository }

func NewService(repositories ...Repository) *Service {
	service := &Service{}
	if len(repositories) > 0 {
		service.repository = repositories[0]
	}
	return service
}
func (s *Service) Get(ctx context.Context) (Example, error) {
	if s.repository == nil {
		return Example{Message: "Olá do Go"}, nil
	}
	message, err := s.repository.Read(ctx)
	return Example{Message: message}, err
}
func (s *Service) Write(ctx context.Context, message string) (Example, error) {
	if s.repository == nil {
		return Example{}, errors.New("repository indisponível")
	}
	err := s.repository.Write(ctx, message)
	return Example{Message: message}, err
}
