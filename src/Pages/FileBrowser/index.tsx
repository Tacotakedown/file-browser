"use client";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Folder,
  File,
  Upload,
  LogOut,
  Download,
  RefreshCcw,
} from "lucide-react";
import { ThemeToggle } from "@/components/darkMode";
import { save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

const FileBrowser = () => {
  const [currentPath, setCurrentPath] = useState<string>("/home/tacotakedown/");
  const [files, setFiles] = useState([]); // TODO: Add type
  const [connectionConfig, setConnectionConfig] = useState({
    host: "169.150.251.167",
    username: "tacotakedown",
    password: "",
    private_key_path: "",
  }); // TODO: Add type
  const [isConnected, setIsConnected] = useState(false);

  const connect = async () => {
    try {
      await invoke("connect", { config: connectionConfig });
      setIsConnected(true);
      loadFiles();
    } catch (error) {
      console.error("Connection failed:", error);
    }
  };

  const logout = async () => {
    try {
      await invoke("disconnect");
    } catch (error) {
      console.error("Logout failed:", error);
    } finally {
      setIsConnected(false);
      setFiles([]);
      setCurrentPath("/home/tacotakedown/");
      setConnectionConfig({
        ...connectionConfig,
        password: "",
        private_key_path: "",
      });
    }
  };

  const loadFiles = async () => {
    try {
      const fileList = await invoke("list_files", { path: currentPath });
      setFiles(fileList);
    } catch (error) {
      console.error("Failed to load files:", error);
    }
  };

  const handleFileClick = (file) => {
    if (file.is_dir) {
      setCurrentPath(`${currentPath}${file.name}/`);
    }
  };

  const handleDownload = async (file) => {
    try {
      const savePath = await save({
        filters: [
          {
            name: file.name,
            extensions: [file.name.split(".").pop() || "*"],
          },
        ],
        defaultPath: file.name,
      });

      if (savePath) {
        const downloadId = Math.random().toString(36).substring(2, 9);
        const unlisten = await listen("download-progress", (event) => {
          const [id, progress] = event.payload;
          if (id === downloadId) {
            console.log("Download Progress: ", progress, "%");
          }
        });

        await invoke("download_file", {
          remotePath: file.path,
          localPath: savePath,
          downloadId: downloadId,
        });

        unlisten();
      }
    } catch (error) {
      console.error("Failed to download file:", error);
    }
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: Async function used globally
  useEffect(() => {
    if (isConnected) {
      loadFiles();
    }
  }, [isConnected]);

  useEffect(() => {
    loadFiles();
  }, [currentPath]);

  return (
    <div className="p-4">
      {!isConnected ? (
        <Card className="w-full max-w-md mx-auto">
          <CardHeader>
            <CardTitle className="flex flex-row items-center w-full justify-between">
              Connect to SSH <ThemeToggle />
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Input
              placeholder="Host"
              value={connectionConfig.host}
              onChange={(e) =>
                setConnectionConfig({
                  ...connectionConfig,
                  host: e.target.value,
                })
              }
            />
            <Input
              placeholder="Username"
              value={connectionConfig.username}
              onChange={(e) =>
                setConnectionConfig({
                  ...connectionConfig,
                  username: e.target.value,
                })
              }
            />
            <Input
              type="password"
              placeholder="Password"
              value={connectionConfig.password}
              onChange={(e) =>
                setConnectionConfig({
                  ...connectionConfig,
                  password: e.target.value,
                })
              }
            />
            <Input
              placeholder="Private Key Path (optional)"
              value={connectionConfig.private_key_path}
              onChange={(e) =>
                setConnectionConfig({
                  ...connectionConfig,
                  private_key_path: e.target.value,
                })
              }
            />
            <Button className="w-full" onClick={connect}>
              Connect
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Input
                value={currentPath}
                onChange={(e) => setCurrentPath(e.target.value)}
                className="w-96"
              />
              <Button onClick={loadFiles} size="icon">
                <RefreshCcw className="w-4 h-4" />
              </Button>
            </div>

            <Button
              onClick={() =>
                setCurrentPath(
                  currentPath.split("/").slice(0, -2).join("/") + "/"
                )
              }
            >
              Up
            </Button>
            <Button variant="destructive" onClick={logout} size="icon">
              <LogOut className="w-4 h-4" />
            </Button>
            <ThemeToggle />
          </div>

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Modified</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {files.map((file) => (
                <TableRow key={file.path}>
                  <TableCell>
                    <div
                      className="flex items-center space-x-2 cursor-pointer"
                      onClick={() => handleFileClick(file)}
                    >
                      {file.is_dir ? (
                        <Folder className="w-4 h-4" />
                      ) : (
                        <File className="w-4 h-4" />
                      )}
                      <span>{file.name}</span>
                    </div>
                  </TableCell>
                  <TableCell>
                    {file.is_dir
                      ? "-"
                      : `${(file.size / 1024 / 1024).toFixed(2)} MB`}
                  </TableCell>
                  <TableCell>
                    {new Date(file.modified * 1000).toLocaleString()}
                  </TableCell>
                  <TableCell>
                    {!file.is_dir && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleDownload(file)}
                      >
                        <Download className="w-4 h-4" />
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
};

export default FileBrowser;
