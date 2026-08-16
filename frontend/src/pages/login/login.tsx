"use client";

import React, { useState } from 'react';
import { FaApple, FaGoogle, FaFacebookF, FaEye, FaEyeSlash } from 'react-icons/fa';
import './login.css';

const Login: React.FC = () => {
  const [identifier, setIdentifier] = useState('');
  const [password, setPassword] = useState('');
  const [isPasswordVisible, setIsPasswordVisible] = useState(false);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!identifier || !password) {
      alert('Please enter both your identifier and password.');
      return;
    }

    // 1. Pull the URL from the environment variable
    const apiUrl = process.env.NEXT_PUBLIC_API_URL;

    // Optional: Safety check in case the .env file wasn't loaded properly
    if (!apiUrl) {
      console.error('API URL is missing from environment variables.');
      return;
    }

    try {
      // 2. Make the API request
      const response = await fetch(`${apiUrl}/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ identifier, password }),
      });

      // 3. Handle server errors (e.g., 401 Unauthorized, 500 Server Error)
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Login failed. Please check your credentials.');
      }

      // 4. Handle success
      const data = await response.json();
      console.log('Login successful!', data);
      
      // TODO: Save your token or redirect the user here

    } catch (error) {
      console.error('Error during login:', error);
      alert(error instanceof Error ? error.message : 'An unexpected error occurred');
    }
  };

  const handleSocialLogin = (provider: string) => {
    // 1. Pull the URL from the environment variable
    const apiUrl = process.env.NEXT_PUBLIC_API_URL;

    if (!apiUrl) {
      console.error('API URL is missing from environment variables.');
      alert('Authentication service is currently unavailable.');
      return;
    }

    // 2. Format the provider name to match your API routes (e.g., "Google" -> "google")
    const formattedProvider = provider.toLowerCase();

    // 3. Redirect the browser to your backend's specific OAuth endpoint
    // Example resulting URL: https://api.yourdomain.com/auth/google
    window.location.href = `${apiUrl}/auth/${formattedProvider}`;
  };

  return (
    <div className="login-wrapper">
      <div className="login-card">
        <div className="login-header">
          <h2>Welcome Back</h2>
          <p>Sign in to continue</p>
        </div>

        <form onSubmit={handleLogin} className="login-form">
          {/* Identifier Input */}
          <div className="input-group">
            <input
              type="text"
              placeholder="Username, Email, or Phone Number"
              value={identifier}
              onChange={(e) => setIdentifier(e.target.value)}
              autoCapitalize="off"
              autoCorrect="off"
              required
            />
          </div>

          {/* Password Input */}
          <div className="input-group password-group">
            <input
              type={isPasswordVisible ? 'text' : 'password'}
              placeholder="Password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
            <button
              type="button"
              className="eye-button"
              onClick={() => setIsPasswordVisible(!isPasswordVisible)}
              aria-label="Toggle password visibility"
            >
              {isPasswordVisible ? <FaEyeSlash /> : <FaEye />}
            </button>
          </div>

          {/* Forgot Password */}
          <div className="forgot-password-container">
            <button type="button" className="forgot-password-text">
              Forgot Password?
            </button>
          </div>

          {/* Submit Button */}
          <button type="submit" className="btn-primary">
            Log In
          </button>
        </form>

        {/* Divider */}
        <div className="divider">
          <span>OR</span>
        </div>

        {/* Social Sign-In Buttons */}
        <div className="social-login-container">
          <button
            type="button"
            className="btn-social btn-apple"
            onClick={() => handleSocialLogin('Apple')}
          >
            <FaApple className="social-icon" />
            Continue with Apple
          </button>

          <button
            type="button"
            className="btn-social btn-google"
            onClick={() => handleSocialLogin('Google')}
          >
            <FaGoogle className="social-icon" />
            Continue with Google
          </button>

          <button
            type="button"
            className="btn-social btn-facebook"
            onClick={() => handleSocialLogin('Facebook')}
          >
            <FaFacebookF className="social-icon" />
            Continue with Facebook
          </button>
        </div>

        {/* Sign Up Link */}
        <div className="login-footer">
          <p>
            Don't have an account? <button className="signup-text">Sign Up</button>
          </p>
        </div>
      </div>
    </div>
  );
};

export default Login;