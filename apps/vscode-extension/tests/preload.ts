import { mock } from 'bun:test';
import { mockVscode } from './mockVscode';

mock.module('vscode', () => ({
  ...mockVscode,
  default: mockVscode,
}));
