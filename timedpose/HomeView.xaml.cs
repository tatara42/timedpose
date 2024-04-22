
using Microsoft.Win32;
using System.IO;
using System.Windows;
using System.Windows.Controls;

using System.Windows.Shapes;

namespace timedpose
{
    /// <summary>
    /// Interaction logic for HomeView.xaml
    /// </summary>
    public partial class HomeView : UserControl
    {
        List<string>? imageFiles;
        string path = getOldFolder();
        int targetTime = 0;
        public HomeView()
        {
            InitializeComponent();
            if (path != "")
            {
                findFiles(path);
            }
        }

        private static string getOldFolder()
        {
            string tempFolder = System.IO.Path.GetTempPath();
            string filePath = System.IO.Path.Combine(tempFolder, "TimedPose.txt");
            if (!File.Exists(filePath))
            {
                File.WriteAllText(filePath, "");
                return "";
            }
            else
            {
                return File.ReadAllText(filePath);
            }
        }
        private void OpenFolderDialogButton_Click(object sender, RoutedEventArgs e)
        {
            // Create a new instance of the open file dialog
            var openFolderDialog = new OpenFolderDialog();

            // Configure the dialog to select directories instead of files
            openFolderDialog.ValidateNames = false;
            openFolderDialog.FolderName = "";



            // Show the dialog and check if the user selected a directory
            if (openFolderDialog.ShowDialog() == true)
            {
                // Get the selected directory path
                string selectedFolderPath = System.IO.Path.GetFullPath(openFolderDialog.FolderName);
                if (selectedFolderPath != null)
                {

                    // Perform actions based on the selected directory path
                    // For example, you can display the selected directory path in a text box or perform some operations on the files in the directory
                    // Replace "YourTextBoxName" with the actual name of your text box control
                    SelectImageFolderLabel.Content = "Selected Image Folder: ";
                    string tempFolder = System.IO.Path.GetTempPath();
                    string filePath = System.IO.Path.Combine(tempFolder, "TimedPose.txt");
                    File.WriteAllText(filePath, selectedFolderPath);

                    findFiles(selectedFolderPath);
                }
            }
        }

        private void setTargetTimeButton_Click(object sender, RoutedEventArgs e)
        {
            targetTime = 30;
        }
        private void setTargetTime60Button_Click(object sender, RoutedEventArgs e)
        {
            targetTime = 60;
        }
        private void setTargetTime120Button_Click(object sender, RoutedEventArgs e)
        {
            targetTime = 120;
        }

        private void StartButton_Click(object sender, RoutedEventArgs e)
        {
            // Perform actions when the start button is clicked
            // For example, you can start a timer or perform some other actions
            // Replace "YourTextBoxName" with the actual name of your text box control
            Window window = Window.GetWindow(this);
            if (imageFiles == null || imageFiles.Count == 0)
            {
                MessageBox.Show("Please select a folder with images first.");
                return;
            }
            window.Content = new PoseView(imageFiles, targetTime);
        }

        private async void findFiles(string folderPath)
        {
            string[] imageExtensions = { ".jpg", ".jpeg", ".png", ".gif", ".bmp" }; // Add more extensions if needed

            try
            {
                // Create a progress bar to display the progress of file search
                var progressBar = new ProgressBar { IsIndeterminate = true };

                // Add the progress bar to your UI (replace "YourGridName" with the name of your Grid control)
                selectImageGrid.Children.Add(progressBar);

                // Run the file search operation asynchronously
                await Task.Run(() =>
                {
                    // Get files in the folder and its subfolders matching the specified extensions
                    imageFiles = Directory.GetFiles(folderPath, "*.*", SearchOption.AllDirectories)
                        .Where(file => imageExtensions.Contains(System.IO.Path.GetExtension(file).ToLower()))
                        .ToList();
                });

                // Display count of image files
                SelectedFolderTextBlock.Text = imageFiles.Count.ToString() + " image files found in the selected folder.";

                // Remove the progress bar from your UI once the search operation is complete
                selectImageGrid.Children.Remove(progressBar);
            }
            catch (Exception ex)
            {
                MessageBox.Show("An error occurred: " + ex.Message);
            }
        }

        private void Button_Click(object sender, RoutedEventArgs e)
        {

        }
    }

}
