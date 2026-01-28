package app

import "open-camera-mouse/internal/config"

type Service struct {
	params config.AllParams
}

func NewService(params config.AllParams) *Service {
	return &Service{params: params}
}

func (s *Service) Params() config.AllParams {
	return s.params
}
